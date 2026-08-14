use super::*;

pub(crate) struct Database {
  connection: Connection,
}

impl Database {
  const MIGRATIONS: &[&str] = &[include_str!("../migrations/0001_initial.sql")];

  const SCHEMA_VERSION: usize = Self::MIGRATIONS.len();

  pub(crate) fn backup(&self, path: &Path) -> Result {
    self
      .connection
      .backup(MAIN_DB, path, None)
      .with_context(|| {
        format!("failed to back up database to `{}`", path.display())
      })
  }

  pub(crate) fn clear(&self) -> Result {
    self.connection.pragma_update(None, "secure_delete", true)?;

    let transaction = self.connection.unchecked_transaction()?;

    transaction.execute("DELETE FROM commands", [])?;
    transaction.execute("DELETE FROM import_sources", [])?;
    transaction.execute("DELETE FROM executions", [])?;

    transaction.commit()?;

    self.connection.execute_batch("VACUUM")?;

    let busy = self.connection.query_row(
      "PRAGMA wal_checkpoint(TRUNCATE)",
      [],
      |row| row.get::<_, bool>(0),
    )?;

    if busy {
      bail!("failed to truncate database write-ahead log because it is busy");
    }

    Ok(())
  }

  #[cfg(test)]
  pub(crate) fn connection(&self) -> &Connection {
    &self.connection
  }

  pub(crate) fn for_each_command(
    &self,
    limit: Option<usize>,
    mut callback: impl FnMut(Command) -> bool,
  ) -> Result {
    let limit = limit
      .map(i64::try_from)
      .transpose()
      .context("command limit exceeds SQLite integer range")?
      .unwrap_or(-1);

    let mut statement = self.connection.prepare(indoc! {
      "
      SELECT text, timestamp_ns, exit_code, directory
      FROM commands
      ORDER BY timestamp_ns DESC, execution_id DESC
      LIMIT ?1
      "
    })?;

    let mut rows = statement.query([limit])?;

    while let Some(row) = rows.next()? {
      if !callback(Command::try_from(row)?) {
        break;
      }
    }

    Ok(())
  }

  pub(crate) fn has_executions(&self) -> Result<bool> {
    self
      .connection
      .query_row("SELECT EXISTS(SELECT 1 FROM executions)", [], |row| {
        row.get(0)
      })
      .map_err(Into::into)
  }

  pub(crate) fn import(
    &self,
    format: &str,
    path: &Path,
    records: impl IntoIterator<Item = Result<Record>>,
    mut progress: impl FnMut(Tally),
  ) -> Result<usize> {
    let path = path.as_os_str().as_encoded_bytes();

    let source_id = Uuid::new_v4().to_string();

    let (source_id, generation) = self.connection.query_row(
      indoc! {
        "
        INSERT INTO import_sources (id, format, path, generation)
        VALUES (?1, ?2, ?3, 1)
        ON CONFLICT (format, path) DO UPDATE SET generation = import_sources.generation + 1
        RETURNING id, generation
        "
      },
      params![source_id, format, path],
      |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
    )?;

    let records = records.into_iter().collect::<Result<Vec<_>>>()?;

    i32::try_from(records.len())
      .context("history contains too many records")?;

    let transaction = Transaction::new_unchecked(
      &self.connection,
      TransactionBehavior::Immediate,
    )?;

    let current_generation = transaction.query_row(
      "SELECT generation FROM import_sources WHERE id = ?1",
      [&source_id],
      |row| row.get::<_, i64>(0),
    )?;

    if current_generation != generation {
      return Ok(0);
    }

    let previous = {
      let mut statement = transaction.prepare(indoc! {
        "
        SELECT fingerprint, execution_id
        FROM source_records
        WHERE source_id = ?1
        ORDER BY position
        "
      })?;

      statement
        .query_map([&source_id], |row| {
          Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
    };

    let mut identifiers = Self::reconcile(&previous, &records)?;

    let inserted = {
      let mut statement = transaction.prepare(indoc! {
        "
        INSERT INTO executions (
          id, command, timestamp_ns, duration_ns, exit_code, directory, session, hostname, shell
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT (id) DO UPDATE SET
          command = excluded.command,
          timestamp_ns = excluded.timestamp_ns,
          duration_ns = excluded.duration_ns,
          exit_code = excluded.exit_code,
          directory = excluded.directory,
          session = excluded.session,
          hostname = excluded.hostname,
          shell = excluded.shell
        WHERE ?10 = 0
        "
      })?;

      let mut inserted = 0;

      for (index, (record, identifier)) in
        records.iter().zip(&mut identifiers).enumerate()
      {
        let new = identifier.is_none();

        let id = identifier.get_or_insert_with(|| Uuid::new_v4().to_string());

        let directory = record.execution.directory()?;

        let changed = statement.execute(params![
          id.as_str(),
          record.execution.command,
          record.execution.timestamp_ns,
          record.execution.duration_ns,
          record.execution.exit_code,
          directory,
          record.execution.session,
          record.execution.hostname,
          record.execution.shell,
          new,
        ])?;

        if new {
          if changed == 0 {
            bail!("generated duplicate execution ID `{id}`");
          }

          inserted += 1;
        }

        progress(Tally {
          inserted,
          processed: index + 1,
        });
      }

      inserted
    };

    transaction.execute(
      "DELETE FROM source_records WHERE source_id = ?1",
      [&source_id],
    )?;

    {
      let mut statement = transaction.prepare(indoc! {
        "
        INSERT INTO source_records (
          source_id, position, fingerprint, execution_id
        ) VALUES (?1, ?2, ?3, ?4)
        "
      })?;

      for (position, (record, identifier)) in
        records.iter().zip(identifiers).enumerate()
      {
        statement.execute(params![
          source_id,
          i64::try_from(position)?,
          record.fingerprint,
          identifier.unwrap(),
        ])?;
      }
    }

    transaction.commit()?;

    Ok(inserted)
  }

  pub(crate) fn insert(&self, execution: &Execution) -> Result<Uuid> {
    let id = Uuid::new_v4();

    let directory = execution.directory()?;

    self.connection.execute(indoc! {
        "
        INSERT INTO executions (
          id, command, timestamp_ns, duration_ns, exit_code, directory, session, hostname, shell
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "
      },
      params![
        id.to_string(),
        execution.command,
        execution.timestamp_ns,
        execution.duration_ns,
        execution.exit_code,
        directory,
        execution.session,
        execution.hostname,
        execution.shell,
      ],
    )?;

    Ok(id)
  }

  pub(crate) fn load() -> Result<Self> {
    #[cfg(unix)]
    let path =
      BaseDirectories::with_prefix("honu").place_data_file("history.db")?;

    #[cfg(windows)]
    let path = {
      let directory = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .context("failed to determine local data directory")?
        .join("honu");

      fs::create_dir_all(&directory)?;

      directory.join("history.db")
    };

    Self::try_from(path.as_path())
  }

  fn open(path: impl AsRef<Path>) -> Result<Self> {
    Self::try_from(Connection::open(path)?)
  }

  pub(crate) fn recent(&self, limit: usize) -> Result<Vec<(Uuid, Execution)>> {
    let limit = i64::try_from(limit)
      .context("execution limit exceeds SQLite integer range")?;

    let mut statement = self.connection.prepare(indoc! {
      "
      SELECT id, command, timestamp_ns, duration_ns, exit_code, directory, session, hostname, shell
      FROM executions
      ORDER BY timestamp_ns DESC, id DESC
      LIMIT ?1
      "
    })?;

    let rows = statement.query_and_then([limit], |row| -> Result<_> {
      let id = row.get::<_, String>("id")?;

      Ok((
        Uuid::parse_str(&id)
          .with_context(|| format!("invalid execution ID `{id}`"))?,
        Execution::try_from(row)?,
      ))
    })?;

    rows.collect()
  }

  fn reconcile(
    previous: &[(Vec<u8>, String)],
    records: &[Record],
  ) -> Result<Vec<Option<String>>> {
    let previous_len = i32::try_from(previous.len())
      .context("previous history contains too many records")?
      .cast_unsigned();

    let records_len = u32::try_from(records.len())
      .context("current history contains too many records")?;

    let mut input = InternedInput::default();

    input.reserve(previous_len, records_len);

    input.update_before(
      previous
        .iter()
        .map(|(fingerprint, _)| fingerprint.as_slice()),
    );

    input
      .update_after(records.iter().map(|record| record.fingerprint.as_slice()));

    let diff = Diff::compute(Algorithm::MyersMinimal, &input);

    let identifiers = (0..records_len)
      .scan(0_u32, |before, after| {
        while *before < previous_len && diff.is_removed(*before) {
          *before += 1;
        }

        let identifier = if diff.is_added(after) {
          None
        } else {
          let identifier = previous[*before as usize].1.clone();
          *before += 1;
          Some(identifier)
        };

        Some(identifier)
      })
      .collect();

    Ok(identifiers)
  }
}

impl TryFrom<Connection> for Database {
  type Error = Error;

  fn try_from(mut connection: Connection) -> Result<Self> {
    connection.busy_timeout(Duration::from_secs(5))?;

    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", true)?;

    let transaction =
      connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

    let version: i64 =
      transaction.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    let schema_version = version;

    let version = match usize::try_from(schema_version) {
      Ok(version) if version <= Self::SCHEMA_VERSION => version,
      _ => bail!(
        "database schema version {schema_version} is unsupported; expected {}",
        Self::SCHEMA_VERSION,
      ),
    };

    for (version, migration) in
      Self::MIGRATIONS.iter().enumerate().skip(version)
    {
      let version = version + 1;

      transaction.execute_batch(migration).with_context(|| {
        format!("failed to apply database migration {version}")
      })?;

      transaction.pragma_update(
        None,
        "user_version",
        i64::try_from(version)?,
      )?;
    }

    transaction.commit()?;

    Ok(Self { connection })
  }
}

impl TryFrom<&Path> for Database {
  type Error = Error;

  fn try_from(path: &Path) -> Result<Self> {
    #[cfg(unix)]
    let directory = path
      .parent()
      .context("database path has no parent directory")?;

    #[cfg(unix)]
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;

    let database = Self::open(path).with_context(|| {
      format!("failed to open database `{}`", path.display())
    })?;

    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;

    Ok(database)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    std::{collections::HashMap, iter},
  };

  #[test]
  fn clear_deletes_all_records() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    database
      .import(
        "foo",
        Path::new("bar"),
        [Ok(Record {
          execution: Execution {
            command: "foo".into(),
            ..Default::default()
          },
          fingerprint: b"bar".to_vec(),
        })],
        |_| {},
      )
      .unwrap();

    database.clear().unwrap();

    assert_eq!(
      database
        .connection()
        .query_row(
          indoc! {
            "
            SELECT
              (SELECT COUNT(*) FROM commands),
              (SELECT COUNT(*) FROM executions),
              (SELECT COUNT(*) FROM import_sources),
              (SELECT COUNT(*) FROM source_records)
            "
          },
          [],
          |row| {
            Ok((
              row.get::<_, i64>(0)?,
              row.get::<_, i64>(1)?,
              row.get::<_, i64>(2)?,
              row.get::<_, i64>(3)?,
            ))
          },
        )
        .unwrap(),
      (0, 0, 0, 0),
    );
  }

  #[test]
  fn clear_purges_database_and_wal() {
    let root = tempfile::tempdir().unwrap();

    let path = root.path().join("foo.db");
    let wal = root.path().join("foo.db-wal");

    let database = Database::open(&path).unwrap();

    let pragma = |name| {
      database
        .connection()
        .pragma_query_value(None, name, |row| row.get::<_, i64>(0))
        .unwrap()
    };

    database
      .connection()
      .execute_batch(indoc! {
        "
        PRAGMA secure_delete = OFF;
        CREATE TABLE foo (bar BLOB);
        INSERT INTO foo VALUES (ZEROBLOB(65536));
        DROP TABLE foo;
        "
      })
      .unwrap();

    assert_eq!(
      (
        pragma("secure_delete"),
        pragma("freelist_count") > 0,
        wal.metadata().unwrap().len() > 0,
      ),
      (0, true, true),
    );

    database.clear().unwrap();

    assert_eq!(
      (
        pragma("secure_delete"),
        pragma("freelist_count"),
        wal.metadata().unwrap().len(),
      ),
      (1, 0, 0),
    );
  }

  #[test]
  fn clear_reports_busy_wal() {
    let root = tempfile::tempdir().unwrap();

    let path = root.path().join("foo.db");

    let database = Database::open(&path).unwrap();

    database
      .insert(&Execution {
        command: "foo".into(),
        ..Default::default()
      })
      .unwrap();

    let reader = Connection::open(path).unwrap();

    reader.execute_batch("BEGIN").unwrap();

    assert_eq!(
      reader
        .query_row("SELECT command FROM executions", [], |row| {
          row.get::<_, String>(0)
        })
        .unwrap(),
      "foo",
    );

    database.connection().busy_timeout(Duration::ZERO).unwrap();

    assert_eq!(
      database.clear().unwrap_err().to_string(),
      "failed to truncate database write-ahead log because it is busy",
    );
  }

  #[test]
  fn for_each_command_orders_and_limits_unique_commands() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    for (command, timestamp_ns, directory, exit_code) in [
      ("foo", 1, "/foo", 1),
      ("bar", 2, "/bar", 2),
      ("foo", 3, "/baz", 3),
    ] {
      database
        .insert(&Execution {
          command: command.into(),
          directory: Some(directory.into()),
          exit_code: Some(exit_code),
          timestamp_ns,
          ..Default::default()
        })
        .unwrap();
    }

    let mut commands = Vec::new();

    database
      .for_each_command(None, |command| {
        commands.push(command);
        true
      })
      .unwrap();

    assert_eq!(
      commands,
      [
        Command {
          directory: Some("/baz".into()),
          exit_code: Some(3),
          text: "foo".into(),
          timestamp_ns: 3,
        },
        Command {
          directory: Some("/bar".into()),
          exit_code: Some(2),
          text: "bar".into(),
          timestamp_ns: 2,
        },
      ],
    );

    database
      .for_each_command(Some(1), |command| {
        commands.push(command);
        true
      })
      .unwrap();

    assert_eq!(
      commands,
      [
        Command {
          directory: Some("/baz".into()),
          exit_code: Some(3),
          text: "foo".into(),
          timestamp_ns: 3,
        },
        Command {
          directory: Some("/bar".into()),
          exit_code: Some(2),
          text: "bar".into(),
          timestamp_ns: 2,
        },
        Command {
          directory: Some("/baz".into()),
          exit_code: Some(3),
          text: "foo".into(),
          timestamp_ns: 3,
        },
      ],
    );

    database
      .for_each_command(Some(0), |command| {
        commands.push(command);
        true
      })
      .unwrap();

    assert_eq!(commands.len(), 3);

    assert!(
      database
        .connection()
        .query_row(
          indoc! {
            "
            EXPLAIN QUERY PLAN
            SELECT text, timestamp_ns, exit_code, directory
            FROM commands
            ORDER BY timestamp_ns DESC, execution_id DESC
            "
          },
          [],
          |row| row.get::<_, String>(3),
        )
        .unwrap()
        .contains("COVERING INDEX commands_timestamp"),
    );
  }

  #[test]
  fn import_inserts_and_is_idempotent() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let records = [
      Record {
        execution: Execution {
          command: "foo".into(),
          timestamp_ns: 1,
          ..Default::default()
        },
        fingerprint: b"foo".to_vec(),
      },
      Record {
        execution: Execution {
          command: "bar".into(),
          timestamp_ns: 2,
          ..Default::default()
        },
        fingerprint: b"bar".to_vec(),
      },
    ];

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          records.iter().cloned().map(Ok),
          |_| {},
        )
        .unwrap(),
      2,
    );

    let imported = database.recent(20).unwrap();

    database
      .connection()
      .execute_batch(indoc! {
        "
        CREATE TABLE command_changes (operation TEXT NOT NULL);
        CREATE TRIGGER commands_insert_change
        AFTER INSERT ON commands
        BEGIN
          INSERT INTO command_changes VALUES ('insert');
        END;
        CREATE TRIGGER commands_update_change
        AFTER UPDATE ON commands
        BEGIN
          INSERT INTO command_changes VALUES ('update');
        END;
        CREATE TRIGGER commands_delete_change
        AFTER DELETE ON commands
        BEGIN
          INSERT INTO command_changes VALUES ('delete');
        END;
        "
      })
      .unwrap();

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          records.into_iter().map(Ok),
          |_| {},
        )
        .unwrap(),
      0,
    );

    assert_eq!(database.recent(20).unwrap(), imported);

    assert_eq!(
      database
        .connection()
        .query_row("SELECT COUNT(*) FROM command_changes", [], |row| {
          row.get::<_, i64>(0)
        })
        .unwrap(),
      0,
    );
  }

  #[test]
  fn import_preserves_repeated_commands_and_metadata() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let first = Execution {
      command: "foo".into(),
      timestamp_ns: 1,
      duration_ns: Some(2),
      exit_code: Some(3),
      directory: Some("/foo".into()),
      session: Some("bar".into()),
      hostname: Some("foo".into()),
      shell: Some("zsh".into()),
    };

    let second = Execution {
      command: "foo".into(),
      timestamp_ns: 4,
      duration_ns: Some(5),
      exit_code: Some(6),
      directory: Some("/bar".into()),
      session: Some("foo".into()),
      hostname: Some("bar".into()),
      shell: Some("zsh".into()),
    };

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          [
            Ok(Record {
              execution: first.clone(),
              fingerprint: b"foo".to_vec(),
            }),
            Ok(Record {
              execution: second.clone(),
              fingerprint: b"bar".to_vec(),
            }),
          ],
          |_| {},
        )
        .unwrap(),
      2,
    );

    assert_eq!(
      database
        .recent(20)
        .unwrap()
        .into_iter()
        .map(|(_, execution)| execution)
        .collect::<Vec<_>>(),
      vec![second, first],
    );
  }

  #[test]
  fn import_reconciles_ordered_source_snapshots() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          [
            Ok(Record {
              execution: Execution {
                command: "foo".into(),
                timestamp_ns: 1,
                ..Default::default()
              },
              fingerprint: b"foo".to_vec(),
            }),
            Ok(Record {
              execution: Execution {
                command: "bar".into(),
                timestamp_ns: 2,
                ..Default::default()
              },
              fingerprint: b"bar".to_vec(),
            }),
          ],
          |_| {},
        )
        .unwrap(),
      2,
    );

    let original = database
      .recent(20)
      .unwrap()
      .into_iter()
      .map(|(id, execution)| (execution.command, (id, execution.timestamp_ns)))
      .collect::<HashMap<_, _>>();

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          [
            Ok(Record {
              execution: Execution {
                command: "baz".into(),
                timestamp_ns: 1,
                ..Default::default()
              },
              fingerprint: b"baz".to_vec(),
            }),
            Ok(Record {
              execution: Execution {
                command: "foo".into(),
                timestamp_ns: 2,
                ..Default::default()
              },
              fingerprint: b"foo".to_vec(),
            }),
            Ok(Record {
              execution: Execution {
                command: "bar".into(),
                timestamp_ns: 3,
                ..Default::default()
              },
              fingerprint: b"bar".to_vec(),
            }),
          ],
          |_| {},
        )
        .unwrap(),
      1,
    );

    let reconciled = database
      .recent(20)
      .unwrap()
      .into_iter()
      .map(|(id, execution)| (execution.command, (id, execution.timestamp_ns)))
      .collect::<HashMap<_, _>>();

    assert_eq!(
      (
        reconciled["foo"].0,
        reconciled["bar"].0,
        reconciled["foo"].1,
        reconciled["bar"].1,
      ),
      (original["foo"].0, original["bar"].0, 2, 3),
    );
  }

  #[test]
  fn import_refreshes_commands() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    for records in [
      [("foo", 3, b"foo".as_slice()), ("bar", 2, b"bar".as_slice())],
      [("foo", 1, b"foo".as_slice()), ("bar", 2, b"bar".as_slice())],
    ] {
      database
        .import(
          "test",
          Path::new("foo"),
          records.map(|(command, timestamp_ns, fingerprint)| {
            Ok(Record {
              execution: Execution {
                command: command.into(),
                timestamp_ns,
                ..Default::default()
              },
              fingerprint: fingerprint.to_vec(),
            })
          }),
          |_| {},
        )
        .unwrap();
    }

    let commands = database
      .connection()
      .prepare(indoc! {
        "
        SELECT text, timestamp_ns
        FROM commands
        ORDER BY timestamp_ns DESC, execution_id DESC
        "
      })
      .unwrap()
      .query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
      })
      .unwrap()
      .collect::<rusqlite::Result<Vec<_>>>()
      .unwrap();

    assert_eq!(commands, [("bar".into(), 2), ("foo".into(), 1)]);
  }

  #[test]
  fn import_retains_truncated_records_and_preserves_new_duplicates() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          [
            Ok(Record {
              execution: Execution {
                command: "foo".into(),
                timestamp_ns: 1,
                ..Default::default()
              },
              fingerprint: b"foo".to_vec(),
            }),
            Ok(Record {
              execution: Execution {
                command: "bar".into(),
                timestamp_ns: 2,
                ..Default::default()
              },
              fingerprint: b"bar".to_vec(),
            }),
          ],
          |_| {},
        )
        .unwrap(),
      2,
    );

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          [Ok(Record {
            execution: Execution {
              command: "bar".into(),
              timestamp_ns: 1,
              ..Default::default()
            },
            fingerprint: b"bar".to_vec(),
          })],
          |_| {},
        )
        .unwrap(),
      0,
    );

    assert_eq!(database.recent(20).unwrap().len(), 2);

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          [
            Ok(Record {
              execution: Execution {
                command: "bar".into(),
                timestamp_ns: 1,
                ..Default::default()
              },
              fingerprint: b"bar".to_vec(),
            }),
            Ok(Record {
              execution: Execution {
                command: "bar".into(),
                timestamp_ns: 2,
                ..Default::default()
              },
              fingerprint: b"bar".to_vec(),
            }),
          ],
          |_| {},
        )
        .unwrap(),
      1,
    );

    assert_eq!(database.recent(20).unwrap().len(), 3);
  }

  #[test]
  fn import_rolls_back_complete_batch_on_constraint_failure() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let error = database
      .import(
        "test",
        Path::new("foo"),
        [
          Ok(Record {
            execution: Execution {
              command: "foo".into(),
              ..Default::default()
            },
            fingerprint: b"foo".to_vec(),
          }),
          Ok(Record {
            execution: Execution {
              command: "bar".into(),
              duration_ns: Some(-1),
              ..Default::default()
            },
            fingerprint: b"bar".to_vec(),
          }),
        ],
        |_| {},
      )
      .unwrap_err();

    let error = error.downcast_ref::<rusqlite::Error>().unwrap();

    assert_eq!(
      (
        error.sqlite_error_code(),
        error.to_string(),
        database.recent(20).unwrap(),
      ),
      (
        Some(rusqlite::ffi::ErrorCode::ConstraintViolation),
        "CHECK constraint failed: duration_ns IS NULL OR duration_ns >= 0"
          .into(),
        Vec::new(),
      ),
    );
  }

  #[test]
  fn import_rolls_back_complete_batch_on_iterator_failure() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let mut progress = Vec::new();

    let error = database
      .import(
        "test",
        Path::new("foo"),
        [
          Ok(Record {
            execution: Execution {
              command: "foo".into(),
              ..Default::default()
            },
            fingerprint: b"foo".to_vec(),
          }),
          Err(Error::msg("bar")),
        ],
        |status| progress.push((status.processed, status.inserted)),
      )
      .unwrap_err();

    assert_eq!(
      (error.to_string(), progress, database.recent(20).unwrap(),),
      ("bar".into(), Vec::new(), Vec::new()),
    );
  }

  #[test]
  fn import_superseded_source_generation_is_discarded() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    assert_eq!(
      database
        .import(
          "test",
          Path::new("foo"),
          iter::once_with(|| {
            database
              .import("test", Path::new("foo"), iter::empty(), |_| {})
              .unwrap();

            Ok(Record {
              execution: Execution {
                command: "foo".into(),
                ..Default::default()
              },
              fingerprint: b"foo".to_vec(),
            })
          }),
          |_| {},
        )
        .unwrap(),
      0,
    );

    assert_eq!(database.recent(20).unwrap(), Vec::new());
  }

  #[test]
  fn insert_stores_every_execution() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let execution = Execution {
      command: "foo".into(),
      timestamp_ns: 1,
      duration_ns: Some(2),
      exit_code: Some(0),
      directory: Some("/foo".into()),
      session: Some("bar".into()),
      hostname: Some("foo".into()),
      shell: Some("bar".into()),
    };

    let (first, second) = (
      database.insert(&execution).unwrap(),
      database.insert(&execution).unwrap(),
    );

    assert_ne!(first, second);

    let mut expected = vec![(first, execution.clone()), (second, execution)];

    expected.sort_by(|(left, _), (right, _)| right.cmp(left));

    assert_eq!(database.recent(2).unwrap(), expected);
  }

  #[test]
  fn negative_duration_is_rejected() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let execution = Execution {
      command: "foo".into(),
      duration_ns: Some(-1),
      ..Default::default()
    };

    let error = database.insert(&execution).unwrap_err();

    let error = error.downcast_ref::<rusqlite::Error>().unwrap();

    assert_eq!(
      (error.sqlite_error_code(), error.to_string()),
      (
        Some(rusqlite::ffi::ErrorCode::ConstraintViolation),
        "CHECK constraint failed: duration_ns IS NULL OR duration_ns >= 0"
          .into(),
      ),
    );
  }

  #[test]
  fn open_creates_schema() {
    let database = Database::open(":memory:").unwrap();

    let database = Database::try_from(database.connection).unwrap();

    assert_eq!(
      database
        .connection()
        .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .unwrap(),
      i64::try_from(Database::SCHEMA_VERSION).unwrap(),
    );
  }

  #[test]
  fn projection_follows_execution_updates_and_deletes() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let first = database
      .insert(&Execution {
        command: "foo".into(),
        timestamp_ns: 1,
        ..Default::default()
      })
      .unwrap();

    let second = database
      .insert(&Execution {
        command: "foo".into(),
        timestamp_ns: 2,
        ..Default::default()
      })
      .unwrap();

    let third = database
      .insert(&Execution {
        command: "bar".into(),
        timestamp_ns: 3,
        ..Default::default()
      })
      .unwrap();

    database
      .connection()
      .execute(indoc! {
          "
          UPDATE executions
          SET command = 'bar', timestamp_ns = 4, exit_code = 5, directory = '/foo'
          WHERE id = ?1
          "
        },
        [second.to_string()],
      )
      .unwrap();

    let commands = || {
      let mut commands = Vec::new();

      database
        .for_each_command(None, |command| {
          commands.push(command);
          true
        })
        .unwrap();

      commands
    };

    assert_eq!(
      commands(),
      [
        Command {
          text: "bar".into(),
          timestamp_ns: 4,
          exit_code: Some(5),
          directory: Some("/foo".into()),
        },
        Command {
          text: "foo".into(),
          timestamp_ns: 1,
          ..Default::default()
        },
      ],
    );

    database
      .connection()
      .execute("DELETE FROM executions WHERE id = ?1", [second.to_string()])
      .unwrap();

    assert_eq!(
      commands(),
      [
        Command {
          text: "bar".into(),
          timestamp_ns: 3,
          ..Default::default()
        },
        Command {
          text: "foo".into(),
          timestamp_ns: 1,
          ..Default::default()
        },
      ],
    );

    database
      .connection()
      .execute("DELETE FROM executions WHERE id = ?1", [first.to_string()])
      .unwrap();

    database
      .connection()
      .execute("DELETE FROM executions WHERE id = ?1", [third.to_string()])
      .unwrap();

    assert_eq!(commands(), []);

    assert!(
      database
        .connection()
        .execute(
          indoc! {
            "
            INSERT INTO commands (
              text, timestamp_ns, execution_id, exit_code, directory
            ) VALUES ('foo', 0, 'bar', NULL, NULL)
            "
          },
          [],
        )
        .is_err(),
    );
  }

  #[test]
  fn recent_orders_and_limits_executions() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    database
      .insert(&Execution {
        command: "foo".into(),
        timestamp_ns: 1,
        ..Default::default()
      })
      .unwrap();

    let id = database
      .insert(&Execution {
        command: "bar".into(),
        timestamp_ns: 2,
        ..Default::default()
      })
      .unwrap();

    assert_eq!(
      database.recent(1).unwrap(),
      vec![(
        id,
        Execution {
          command: "bar".into(),
          timestamp_ns: 2,
          ..Default::default()
        },
      )],
    );

    assert_eq!(database.recent(0).unwrap(), Vec::new());
  }

  #[cfg(target_pointer_width = "64")]
  #[test]
  fn recent_rejects_large_limit() {
    let database =
      Database::try_from(Connection::open_in_memory().unwrap()).unwrap();

    let Err(error) = database.recent(usize::MAX) else {
      panic!("expected large limit to fail")
    };

    assert_eq!(
      error.to_string(),
      "execution limit exceeds SQLite integer range",
    );
  }

  #[test]
  fn reconcile_minimizes_new_records() {
    let mut state = 0_u32;

    let mut next = || {
      state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
      vec![(state >> 24) as u8]
    };

    let fingerprints = (0..30_000).map(|_| next()).collect::<Vec<_>>();

    let previous = fingerprints
      .iter()
      .enumerate()
      .map(|(index, fingerprint)| (fingerprint.clone(), index.to_string()))
      .collect::<Vec<_>>();

    let records = fingerprints[3_000..]
      .iter()
      .cloned()
      .chain((0..300).map(|_| next()))
      .map(|fingerprint| Record {
        execution: Execution::default(),
        fingerprint,
      })
      .collect::<Vec<_>>();

    assert_eq!(
      Database::reconcile(&previous, &records)
        .unwrap()
        .into_iter()
        .filter(Option::is_none)
        .count(),
      300,
    );
  }

  #[test]
  fn try_from_path_creates_private_database() {
    let root = tempfile::tempdir().unwrap();

    let path = root.path().join("foo/history.db");

    fs::create_dir_all(path.parent().unwrap()).unwrap();

    let database = Database::try_from(path.as_path()).unwrap();

    assert_eq!(
      (
        path.is_file(),
        database
          .connection()
          .pragma_query_value(None, "journal_mode", |row| {
            row.get::<_, String>(0)
          })
          .unwrap(),
        database
          .connection()
          .pragma_query_value(None, "busy_timeout", |row| {
            row.get::<_, i64>(0)
          })
          .unwrap(),
      ),
      (true, "wal".into(), 5000),
    );

    #[cfg(unix)]
    assert_eq!(
      (
        fs::metadata(path.parent().unwrap())
          .unwrap()
          .permissions()
          .mode()
          & 0o777,
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
      ),
      (0o700, 0o600),
    );
  }

  #[test]
  fn unsupported_schema_is_rejected() {
    let connection = Connection::open_in_memory().unwrap();

    connection.execute_batch("PRAGMA user_version = 5").unwrap();

    let Err(error) = Database::try_from(connection) else {
      panic!("expected unsupported schema to fail")
    };

    assert_eq!(
      error.to_string(),
      "database schema version 5 is unsupported; expected 1",
    );
  }
}
