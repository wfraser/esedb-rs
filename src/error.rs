// Extensible Storage Engine database library for Rust
// Copyright 2016-2019 by William R. Fraser

#![allow(non_upper_case_globals)]

use winapi::ctypes::{c_ulong, c_void};
use winapi::um::esent::*;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::mem;
use std::sync::Once;

static mut ERROR_TEXT: *const BTreeMap<JET_ERR, &'static str> = 0 as *const BTreeMap<_, _>;
static ERROR_TEXT_ONCE: Once = Once::new();

fn get_text(code: JET_ERR) -> Option<&'static str> {
    unsafe {
        // It would be cool if this could be done at compile time...
        ERROR_TEXT_ONCE.call_once(|| {
            let mut map = Box::new(BTreeMap::new());
            make_error_text_map(&mut map);
            ERROR_TEXT = Box::into_raw(map);
        });
        match (*ERROR_TEXT).get(&code) {
            Some(s) => Some(s),
            None => None,
        }
    }
}

#[test]
fn test_err_text() {
    assert_eq!(Some("read/write access is not supported on compressed files"), get_text(-4005));
}

#[derive(Debug)]
pub struct JetError {
    pub code: JET_ERR,
    pub text: &'static str,
}

impl From<JET_ERR> for JetError {
    fn from(code: JET_ERR) -> JetError {
        JetError {
            code,
            text: get_text(code).unwrap_or("[no error text]"),
        }
    }
}

impl Display for JetError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        unsafe {
            let mut info = mem::zeroed::<JET_ERRINFOBASIC_W>();
            info.cbStruct = mem::size_of::<JET_ERRINFOBASIC_W>() as c_ulong;
            let err = JetGetErrorInfoW(
                &self.code as *const _ as *const c_void,
                &mut info as *mut _ as *mut c_void,
                info.cbStruct,
                JET_ErrorInfoSpecificErr,
                JET_bitNil,
            );
            if err == JET_errSuccess {
                let cat = match info.errcatMostSpecific {
                    JET_errcatUnknown => "unknown",
                    JET_errcatError => "[general error]",
                    JET_errcatOperation => "operational",
                    JET_errcatFatal => "fatal",
                    JET_errcatIO => "I/O",
                    JET_errcatResource => "resource",
                    JET_errcatMemory => "memory",
                    JET_errcatQuota => "quota",
                    JET_errcatDisk => "disk space",
                    JET_errcatData => "data",
                    JET_errcatCorruption => "data corruption",
                    JET_errcatInconsistent => "data inconsistency",
                    JET_errcatFragmentation => "fragmentation",
                    JET_errcatApi => "API",
                    JET_errcatUsage => "programmer",
                    JET_errcatState => "database state",
                    JET_errcatObsolete => "[obsolete error]",
                    _ => unreachable!(),
                };
                write!(fmt, "ESE DB {} error: {} (code {})", cat, self.text, self.code)
            } else {
                write!(fmt, "ESE DB error: {} (code {}) (failed to get error type because code {})",
                    self.text, self.code, err)
            }
        }
    }
}

impl Error for JetError {
    fn description(&self) -> &str {
        "ESE DB Error"
    }
}

fn make_error_text_map(map: &mut BTreeMap<JET_ERR, &'static str>) {
    map.insert(JET_errSuccess, "Successful Operation");
    map.insert(JET_wrnNyi, "Function Not Yet Implemented");
    map.insert(JET_errRfsFailure, "Resource Failure Simulator failure");
    map.insert(JET_errRfsNotArmed, "Resource Failure Simulator not initialized");
    map.insert(JET_errFileClose, "Could not close file");
    map.insert(JET_errOutOfThreads, "Could not start thread");
    map.insert(JET_errTooManyIO, "System busy due to too many IOs");
    map.insert(JET_errTaskDropped, "A requested async task could not be executed");
    map.insert(JET_errInternalError, "Fatal internal error");
    map.insert(JET_errDisabledFunctionality, "You are running MinESE, that does not have all features compiled in.  This functionality is only supported in a full version of ESE.");
    map.insert(JET_errUnloadableOSFunctionality, "The desired OS functionality could not be located and loaded / linked.");
    map.insert(JET_errDatabaseBufferDependenciesCorrupted, "Buffer dependencies improperly set. Recovery failure");
    map.insert(JET_wrnRemainingVersions, "The version store is still active");
    map.insert(JET_errPreviousVersion, "Version already existed. Recovery failure");
    map.insert(JET_errPageBoundary, "Reached Page Boundary");
    map.insert(JET_errKeyBoundary, "Reached Key Boundary");
    map.insert(JET_errBadPageLink, "Database corrupted");
    map.insert(JET_errBadBookmark, "Bookmark has no corresponding address in database");
    map.insert(JET_errNTSystemCallFailed, "A call to the operating system failed");
    map.insert(JET_errBadParentPageLink, "Database corrupted");
    map.insert(JET_errSPAvailExtCacheOutOfSync, "AvailExt cache doesn't match btree");
    map.insert(JET_errSPAvailExtCorrupted, "AvailExt space tree is corrupt");
    map.insert(JET_errSPAvailExtCacheOutOfMemory, "Out of memory allocating an AvailExt cache node");
    map.insert(JET_errSPOwnExtCorrupted, "OwnExt space tree is corrupt");
    map.insert(JET_errDbTimeCorrupted, "Dbtime on current page is greater than global database dbtime");
    map.insert(JET_wrnUniqueKey, "seek on non-unique index yielded a unique key");
    map.insert(JET_errKeyTruncated, "key truncated on index that disallows key truncation");
    map.insert(JET_errDatabaseLeakInSpace, "Some database pages have become unreachable even from the avail tree, only an offline defragmentation can return the lost space.");
    map.insert(JET_errBadEmptyPage, "Database corrupted. Searching an unexpectedly empty page.");
    map.insert(JET_wrnSeparateLongValue, "Column is a separated long-value");
    map.insert(JET_errKeyTooBig, "Key is too large");
    map.insert(JET_errCannotSeparateIntrinsicLV, "illegal attempt to separate an LV which must be intrinsic");
    map.insert(JET_errSeparatedLongValue, "Operation not supported on separated long-value");
    map.insert(JET_errMustBeSeparateLongValue, "Can only preread long value columns that can be separate, e.g. not size constrained so that they are fixed or variable columns");
    map.insert(JET_errInvalidPreread, "Cannot preread long values when current index secondary");
    map.insert(JET_errInvalidLoggedOperation, "Logged operation cannot be redone");
    map.insert(JET_errLogFileCorrupt, "Log file is corrupt");
    map.insert(JET_errNoBackupDirectory, "No backup directory given");
    map.insert(JET_errBackupDirectoryNotEmpty, "The backup directory is not emtpy");
    map.insert(JET_errBackupInProgress, "Backup is active already");
    map.insert(JET_errRestoreInProgress, "Restore in progress");
    map.insert(JET_errMissingPreviousLogFile, "Missing the log file for check point");
    map.insert(JET_errLogWriteFail, "Failure writing to log file");
    map.insert(JET_errLogDisabledDueToRecoveryFailure, "Try to log something after recovery faild");
    map.insert(JET_errCannotLogDuringRecoveryRedo, "Try to log something during recovery redo");
    map.insert(JET_errLogGenerationMismatch, "Name of logfile does not match internal generation number");
    map.insert(JET_errBadLogVersion, "Version of log file is not compatible with Jet version");
    map.insert(JET_errInvalidLogSequence, "Timestamp in next log does not match expected");
    map.insert(JET_errLoggingDisabled, "Log is not active");
    map.insert(JET_errLogBufferTooSmall, "Log buffer is too small for recovery");
    map.insert(JET_errLogSequenceEnd, "Maximum log file number exceeded");
    map.insert(JET_errNoBackup, "No backup in progress");
    map.insert(JET_errInvalidBackupSequence, "Backup call out of sequence");
    map.insert(JET_errBackupNotAllowedYet, "Cannot do backup now");
    map.insert(JET_errDeleteBackupFileFail, "Could not delete backup file");
    map.insert(JET_errMakeBackupDirectoryFail, "Could not make backup temp directory");
    map.insert(JET_errInvalidBackup, "Cannot perform incremental backup when circular logging enabled");
    map.insert(JET_errRecoveredWithErrors, "Restored with errors");
    map.insert(JET_errMissingLogFile, "Current log file missing");
    map.insert(JET_errLogDiskFull, "Log disk full");
    map.insert(JET_errBadLogSignature, "Bad signature for a log file");
    map.insert(JET_errBadDbSignature, "Bad signature for a db file");
    map.insert(JET_errBadCheckpointSignature, "Bad signature for a checkpoint file");
    map.insert(JET_errCheckpointCorrupt, "Checkpoint file not found or corrupt");
    map.insert(JET_errMissingPatchPage, "Patch file page not found during recovery");
    map.insert(JET_errBadPatchPage, "Patch file page is not valid");
    map.insert(JET_errRedoAbruptEnded, "Redo abruptly ended due to sudden failure in reading logs from log file");
    map.insert(JET_errPatchFileMissing, "Hard restore detected that patch file is missing from backup set");
    map.insert(JET_errDatabaseLogSetMismatch, "Database does not belong with the current set of log files");
    map.insert(JET_errDatabaseStreamingFileMismatch, "Database and streaming file do not match each other");
    map.insert(JET_errLogFileSizeMismatch, "actual log file size does not match JET_paramLogFileSize");
    map.insert(JET_errCheckpointFileNotFound, "Could not locate checkpoint file");
    map.insert(JET_errRequiredLogFilesMissing, "The required log files for recovery is missing.");
    map.insert(JET_errSoftRecoveryOnBackupDatabase, "Soft recovery is intended on a backup database. Restore should be used instead");
    map.insert(JET_errLogFileSizeMismatchDatabasesConsistent, "databases have been recovered, but the log file size used during recovery does not match JET_paramLogFileSize");
    map.insert(JET_errLogSectorSizeMismatch, "the log file sector size does not match the current volume's sector size");
    map.insert(JET_errLogSectorSizeMismatchDatabasesConsistent, "databases have been recovered, but the log file sector size (used during recovery) does not match the current volume's sector size");
    map.insert(JET_errLogSequenceEndDatabasesConsistent, "databases have been recovered, but all possible log generations in the current sequence are used; delete all log files and the checkpoint file and backup the databases before continuing");
    map.insert(JET_errStreamingDataNotLogged, "Illegal attempt to replay a streaming file operation where the data wasn't logged. Probably caused by an attempt to roll-forward with circular logging enabled");
    map.insert(JET_errDatabaseDirtyShutdown, "Database was not shutdown cleanly. Recovery must first be run to properly complete database operations for the previous shutdown.");
    map.insert(JET_errDatabaseInconsistent, "OBSOLETE");
    map.insert(JET_errConsistentTimeMismatch, "Database last consistent time unmatched");
    map.insert(JET_errDatabasePatchFileMismatch, "Patch file is not generated from this backup");
    map.insert(JET_errEndingRestoreLogTooLow, "The starting log number too low for the restore");
    map.insert(JET_errStartingRestoreLogTooHigh, "The starting log number too high for the restore");
    map.insert(JET_errGivenLogFileHasBadSignature, "Restore log file has bad signature");
    map.insert(JET_errGivenLogFileIsNotContiguous, "Restore log file is not contiguous");
    map.insert(JET_errMissingRestoreLogFiles, "Some restore log files are missing");
    map.insert(JET_wrnExistingLogFileHasBadSignature, "Existing log file has bad signature");
    map.insert(JET_wrnExistingLogFileIsNotContiguous, "Existing log file is not contiguous");
    map.insert(JET_errMissingFullBackup, "The database missed a previous full backup before incremental backup");
    map.insert(JET_errBadBackupDatabaseSize, "The backup database size is not in 4k");
    map.insert(JET_errDatabaseAlreadyUpgraded, "Attempted to upgrade a database that is already current");
    map.insert(JET_errDatabaseIncompleteUpgrade, "Attempted to use a database which was only partially converted to the current format -- must restore from backup");
    map.insert(JET_wrnSkipThisRecord, "INTERNAL ERROR");
    map.insert(JET_errMissingCurrentLogFiles, "Some current log files are missing for continuous restore");
    map.insert(JET_errDbTimeTooOld, "dbtime on page smaller than dbtimeBefore in record");
    map.insert(JET_errDbTimeTooNew, "dbtime on page in advance of the dbtimeBefore in record");
    map.insert(JET_errMissingFileToBackup, "Some log or patch files are missing during backup");
    map.insert(JET_errLogTornWriteDuringHardRestore, "torn-write was detected in a backup set during hard restore");
    map.insert(JET_errLogTornWriteDuringHardRecovery, "torn-write was detected during hard recovery (log was not part of a backup set)");
    map.insert(JET_errLogCorruptDuringHardRestore, "corruption was detected in a backup set during hard restore");
    map.insert(JET_errLogCorruptDuringHardRecovery, "corruption was detected during hard recovery (log was not part of a backup set)");
    map.insert(JET_errMustDisableLoggingForDbUpgrade, "Cannot have logging enabled while attempting to upgrade db");
    map.insert(JET_errBadRestoreTargetInstance, "TargetInstance specified for restore is not found or log files don't match");
    map.insert(JET_wrnTargetInstanceRunning, "TargetInstance specified for restore is running");
    map.insert(JET_errRecoveredWithoutUndo, "Soft recovery successfully replayed all operations, but the Undo phase of recovery was skipped");
    map.insert(JET_errDatabasesNotFromSameSnapshot, "Databases to be restored are not from the same shadow copy backup");
    map.insert(JET_errSoftRecoveryOnSnapshot, "Soft recovery on a database from a shadow copy backup set");
    map.insert(JET_errCommittedLogFilesMissing, "One or more logs that were committed to this database, are missing.  These log files are required to maintain durable ACID semantics, but not required to maintain consistency if the JET_bitReplayIgnoreLostLogs bit is specified during recovery.");
    map.insert(JET_errSectorSizeNotSupported, "The physical sector size reported by the disk subsystem, is unsupported by ESE for a specific file type.");
    map.insert(JET_errRecoveredWithoutUndoDatabasesConsistent, "Soft recovery successfully replayed all operations and intended to skip the Undo phase of recovery, but the Undo phase was not required");
    map.insert(JET_wrnCommittedLogFilesLost, "One or more logs that were committed to this database, were not recovered.  The database is still clean/consistent, as though the lost log's transactions were committed lazily (and lost).");
    map.insert(JET_errCommittedLogFileCorrupt, "One or more logs were found to be corrupt during recovery.  These log files are required to maintain durable ACID semantics, but not required to maintain consistency if the JET_bitIgnoreLostLogs bit and JET_paramDeleteOutOfRangeLogs is specified during recovery.");
    map.insert(JET_wrnCommittedLogFilesRemoved, "One or more logs that were committed to this database, were no recovered.  The database is still clean/consistent, as though the corrupted log's transactions were committed lazily (and lost).");
    map.insert(JET_wrnFinishWithUndo, "Signal used by clients to indicate JetInit() finished with undo");
    map.insert(JET_wrnDatabaseRepaired, "Database corruption has been repaired");
    map.insert(JET_errUnicodeTranslationBufferTooSmall, "Unicode translation buffer too small");
    map.insert(JET_errUnicodeTranslationFail, "Unicode normalization failed");
    map.insert(JET_errUnicodeNormalizationNotSupported, "OS does not provide support for Unicode normalisation (and no normalisation callback was specified)");
    map.insert(JET_errUnicodeLanguageValidationFailure, "Can not validate the language");
    map.insert(JET_errExistingLogFileHasBadSignature, "Existing log file has bad signature");
    map.insert(JET_errExistingLogFileIsNotContiguous, "Existing log file is not contiguous");
    map.insert(JET_errLogReadVerifyFailure, "Checksum error in log file during backup");
    map.insert(JET_errCheckpointDepthTooDeep, "	too many outstanding generations between checkpoint and current generation");
    map.insert(JET_errRestoreOfNonBackupDatabase, "hard recovery attempted on a database that wasn't a backup database");
    map.insert(JET_errLogFileNotCopied, "log truncation attempted but not all required logs were copied");
    map.insert(JET_errTransactionTooLong, "Too many outstanding generations between JetBeginTransaction and current generation.");
    map.insert(JET_errEngineFormatVersionNoLongerSupportedTooLow, "The specified JET_ENGINEFORMATVERSION value is too low to be supported by this version of ESE.");
    map.insert(JET_errEngineFormatVersionNotYetImplementedTooHigh, "The specified JET_ENGINEFORMATVERSION value is too high, higher than this version of ESE knows about.");
    map.insert(JET_errEngineFormatVersionParamTooLowForRequestedFeature, "Thrown by a format feature (not at JetSetSystemParameter) if the client requests a feature that requires a version higher than that set for the JET_paramEngineFormatVersion.");
    map.insert(JET_errEngineFormatVersionSpecifiedTooLowForLogVersion, "The specified JET_ENGINEFORMATVERSION is set too low for this log stream, the log files have already been upgraded to a higher version.  A higher JET_ENGINEFORMATVERSION value must be set in the param.");
    map.insert(JET_errEngineFormatVersionSpecifiedTooLowForDatabaseVersion, "The specified JET_ENGINEFORMATVERSION is set too low for this database file, the database file has already been upgraded to a higher version.  A higher JET_ENGINEFORMATVERSION value must be set in the param.");
    map.insert(JET_errBackupAbortByServer, "Backup was aborted by server by calling JetTerm with JET_bitTermStopBackup or by calling JetStopBackup");
    map.insert(JET_errInvalidGrbit, "Invalid flags parameter");
    map.insert(JET_errTermInProgress, "Termination in progress");
    map.insert(JET_errFeatureNotAvailable, "API not supported");
    map.insert(JET_errInvalidName, "Invalid name");
    map.insert(JET_errInvalidParameter, "Invalid API parameter");
    map.insert(JET_wrnColumnNull, "Column is NULL-valued");
    map.insert(JET_wrnBufferTruncated, "Buffer too small for data");
    map.insert(JET_wrnDatabaseAttached, "Database is already attached");
    map.insert(JET_errDatabaseFileReadOnly, "Tried to attach a read-only database file for read/write operations");
    map.insert(JET_wrnSortOverflow, "Sort does not fit in memory");
    map.insert(JET_errInvalidDatabaseId, "Invalid database id");
    map.insert(JET_errOutOfMemory, "Out of Memory");
    map.insert(JET_errOutOfDatabaseSpace, "Maximum database size reached");
    map.insert(JET_errOutOfCursors, "Out of table cursors");
    map.insert(JET_errOutOfBuffers, "Out of database page buffers");
    map.insert(JET_errTooManyIndexes, "Too many indexes");
    map.insert(JET_errTooManyKeys, "Too many columns in an index");
    map.insert(JET_errRecordDeleted, "Record has been deleted");
    map.insert(JET_errReadVerifyFailure, "Checksum error on a database page");
    map.insert(JET_errPageNotInitialized, "Blank database page");
    map.insert(JET_errOutOfFileHandles, "Out of file handles");
    map.insert(JET_errDiskReadVerificationFailure, "The OS returned ERROR_CRC from file IO");
    map.insert(JET_errDiskIO, "Disk IO error");
    map.insert(JET_errInvalidPath, "Invalid file path");
    map.insert(JET_errInvalidSystemPath, "Invalid system path");
    map.insert(JET_errInvalidLogDirectory, "Invalid log directory");
    map.insert(JET_errRecordTooBig, "Record larger than maximum size");
    map.insert(JET_errTooManyOpenDatabases, "Too many open databases");
    map.insert(JET_errInvalidDatabase, "Not a database file");
    map.insert(JET_errNotInitialized, "Database engine not initialized");
    map.insert(JET_errAlreadyInitialized, "Database engine already initialized");
    map.insert(JET_errInitInProgress, "Database engine is being initialized");
    map.insert(JET_errFileAccessDenied, "Cannot access file, the file is locked or in use");
    map.insert(JET_errBufferTooSmall, "Buffer is too small");
    map.insert(JET_wrnSeekNotEqual, "Exact match not found during seek");
    map.insert(JET_errTooManyColumns, "Too many columns defined");
    map.insert(JET_errContainerNotEmpty, "Container is not empty");
    map.insert(JET_errInvalidFilename, "Filename is invalid");
    map.insert(JET_errInvalidBookmark, "Invalid bookmark");
    map.insert(JET_errColumnInUse, "Column used in an index");
    map.insert(JET_errInvalidBufferSize, "Data buffer doesn't match column size");
    map.insert(JET_errColumnNotUpdatable, "Cannot set column value");
    map.insert(JET_errIndexInUse, "Index is in use");
    map.insert(JET_errLinkNotSupported, "Link support unavailable");
    map.insert(JET_errNullKeyDisallowed, "Null keys are disallowed on index");
    map.insert(JET_errNotInTransaction, "Operation must be within a transaction");
    map.insert(JET_wrnNoErrorInfo, "No extended error information");
    map.insert(JET_errMustRollback, "Transaction must rollback because failure of unversioned update");
    map.insert(JET_wrnNoIdleActivity, "No idle activity occured");
    map.insert(JET_errTooManyActiveUsers, "Too many active database users");
    map.insert(JET_errInvalidCountry, "Invalid or unknown country/region code");
    map.insert(JET_errInvalidLanguageId, "Invalid or unknown language id");
    map.insert(JET_errInvalidCodePage, "Invalid or unknown code page");
    map.insert(JET_errInvalidLCMapStringFlags, "Invalid flags for LCMapString()");
    map.insert(JET_errVersionStoreEntryTooBig, "Attempted to create a version store entry (RCE) larger than a version bucket");
    map.insert(JET_errVersionStoreOutOfMemoryAndCleanupTimedOut, "Version store out of memory (and cleanup attempt failed to complete)");
    map.insert(JET_wrnNoWriteLock, "No write lock at transaction level 0");
    map.insert(JET_wrnColumnSetNull, "Column set to NULL-value");
    map.insert(JET_errVersionStoreOutOfMemory, "Version store out of memory (cleanup already attempted)");
    map.insert(JET_errCannotIndex, "Cannot index escrow column");
    map.insert(JET_errRecordNotDeleted, "Record has not been deleted");
    map.insert(JET_errTooManyMempoolEntries, "Too many mempool entries requested");
    map.insert(JET_errOutOfObjectIDs, "Out of btree ObjectIDs (perform offline defrag to reclaim freed/unused ObjectIds)");
    map.insert(JET_errOutOfLongValueIDs, "Long-value ID counter has reached maximum value. (perform offline defrag to reclaim free/unused LongValueIDs)");
    map.insert(JET_errOutOfAutoincrementValues, "Auto-increment counter has reached maximum value (offline defrag WILL NOT be able to reclaim free/unused Auto-increment values).");
    map.insert(JET_errOutOfDbtimeValues, "Dbtime counter has reached maximum value (perform offline defrag to reclaim free/unused Dbtime values)");
    map.insert(JET_errOutOfSequentialIndexValues, "Sequential index counter has reached maximum value (perform offline defrag to reclaim free/unused SequentialIndex values)");
    map.insert(JET_errRunningInOneInstanceMode, "Multi-instance call with single-instance mode enabled");
    map.insert(JET_errRunningInMultiInstanceMode, "Single-instance call with multi-instance mode enabled");
    map.insert(JET_errSystemParamsAlreadySet, "Global system parameters have already been set");
    map.insert(JET_errSystemPathInUse, "System path already used by another database instance");
    map.insert(JET_errLogFilePathInUse, "Logfile path already used by another database instance");
    map.insert(JET_errTempPathInUse, "Temp path already used by another database instance");
    map.insert(JET_errInstanceNameInUse, "Instance Name already in use");
    map.insert(JET_errSystemParameterConflict, "Global system parameters have already been set, but to a conflicting or disagreeable state to the specified values.");
    map.insert(JET_errInstanceUnavailable, "This instance cannot be used because it encountered a fatal error");
    map.insert(JET_errDatabaseUnavailable, "This database cannot be used because it encountered a fatal error");
    map.insert(JET_errInstanceUnavailableDueToFatalLogDiskFull, "This instance cannot be used because it encountered a log-disk-full error performing an operation (likely transaction rollback) that could not tolerate failure");
    map.insert(JET_errInvalidSesparamId, "This JET_sesparam* identifier is not known to the ESE engine.");
    map.insert(JET_errOutOfSessions, "Out of sessions");
    map.insert(JET_errWriteConflict, "Write lock failed due to outstanding write lock");
    map.insert(JET_errTransTooDeep, "Transactions nested too deeply");
    map.insert(JET_errInvalidSesid, "Invalid session handle");
    map.insert(JET_errWriteConflictPrimaryIndex, "Update attempted on uncommitted primary index");
    map.insert(JET_errInTransaction, "Operation not allowed within a transaction");
    map.insert(JET_errRollbackRequired, "Must rollback current transaction -- cannot commit or begin a new one");
    map.insert(JET_errTransReadOnly, "Read-only transaction tried to modify the database");
    map.insert(JET_errSessionWriteConflict, "Attempt to replace the same record by two diffrerent cursors in the same session");
    map.insert(JET_errRecordTooBigForBackwardCompatibility, "record would be too big if represented in a database format from a previous version of Jet");
    map.insert(JET_errCannotMaterializeForwardOnlySort, "The temp table could not be created due to parameters that conflict with JET_bitTTForwardOnly");
    map.insert(JET_errSesidTableIdMismatch, "This session handle can't be used with this table id");
    map.insert(JET_errInvalidInstance, "Invalid instance handle");
    map.insert(JET_errDirtyShutdown, "The instance was shutdown successfully but all the attached databases were left in a dirty state by request via JET_bitTermDirty");
    map.insert(JET_errReadPgnoVerifyFailure, "The database page read from disk had the wrong page number.");
    map.insert(JET_errReadLostFlushVerifyFailure, "The database page read from disk had a previous write not represented on the page.");
    map.insert(JET_errFileSystemCorruption, "File system operation failed with an error indicating the file system is corrupt.");
    map.insert(JET_wrnShrinkNotPossible, "Database file could not be shrunk because there is not enough internal free space available or there is unmovable data present.");
    map.insert(JET_errRecoveryVerifyFailure, "One or more database pages read from disk during recovery do not match the expected state.");
    map.insert(JET_errFilteredMoveNotSupported, "Attempted to provide a filter to JetSetCursorFilter() in an unsupported scenario.");
    map.insert(JET_errDatabaseDuplicate, "Database already exists");
    map.insert(JET_errDatabaseInUse, "Database in use");
    map.insert(JET_errDatabaseNotFound, "No such database");
    map.insert(JET_errDatabaseInvalidName, "Invalid database name");
    map.insert(JET_errDatabaseInvalidPages, "Invalid number of pages");
    map.insert(JET_errDatabaseCorrupted, "Non database file or corrupted db");
    map.insert(JET_errDatabaseLocked, "Database exclusively locked");
    map.insert(JET_errCannotDisableVersioning, "Cannot disable versioning for this database");
    map.insert(JET_errInvalidDatabaseVersion, "Database engine is incompatible with database");
    map.insert(JET_errDatabase200Format, "The database is in an older (200) format");
    map.insert(JET_errDatabase400Format, "The database is in an older (400) format");
    map.insert(JET_errDatabase500Format, "The database is in an older (500) format");
    map.insert(JET_errPageSizeMismatch, "The database page size does not match the engine");
    map.insert(JET_errTooManyInstances, "Cannot start any more database instances");
    map.insert(JET_errDatabaseSharingViolation, "A different database instance is using this database");
    map.insert(JET_errAttachedDatabaseMismatch, "An outstanding database attachment has been detected at the start or end of recovery, but database is missing or does not match attachment info");
    map.insert(JET_errDatabaseInvalidPath, "Specified path to database file is illegal");
    map.insert(JET_errDatabaseIdInUse, "A database is being assigned an id already in use");
    map.insert(JET_errForceDetachNotAllowed, "Force Detach allowed only after normal detach errored out");
    map.insert(JET_errCatalogCorrupted, "Corruption detected in catalog");
    map.insert(JET_errPartiallyAttachedDB, "Database is partially attached. Cannot complete attach operation");
    map.insert(JET_errDatabaseSignInUse, "Database with same signature in use");
    map.insert(JET_errDatabaseCorruptedNoRepair, "Corrupted db but repair not allowed");
    map.insert(JET_errInvalidCreateDbVersion, "recovery tried to replay a database creation, but the database was originally created with an incompatible (likely older) version of the database engine");
    map.insert(JET_errDatabaseNotReady, "Recovery on this database has not yet completed enough to permit access.");
    map.insert(JET_errDatabaseAttachedForRecovery, "Database is attached but only for recovery.  It must be explicitly attached before it can be opened. ");
    map.insert(JET_errTransactionsNotReadyDuringRecovery, "Recovery has not seen any Begin0/Commit0 records and so does not know what trxBegin0 to assign to this transaction");
    map.insert(JET_wrnTableEmpty, "Opened an empty table");
    map.insert(JET_errTableLocked, "Table is exclusively locked");
    map.insert(JET_errTableDuplicate, "Table already exists");
    map.insert(JET_errTableInUse, "Table is in use, cannot lock");
    map.insert(JET_errObjectNotFound, "No such table or object");
    map.insert(JET_errDensityInvalid, "Bad file/index density");
    map.insert(JET_errTableNotEmpty, "Table is not empty");
    map.insert(JET_errInvalidTableId, "Invalid table id");
    map.insert(JET_errTooManyOpenTables, "Cannot open any more tables (cleanup already attempted)");
    map.insert(JET_errIllegalOperation, "Oper. not supported on table");
    map.insert(JET_errTooManyOpenTablesAndCleanupTimedOut, "Cannot open any more tables (cleanup attempt failed to complete)");
    map.insert(JET_errObjectDuplicate, "Table or object name in use");
    map.insert(JET_errInvalidObject, "Object is invalid for operation");
    map.insert(JET_errCannotDeleteTempTable, "Use CloseTable instead of DeleteTable to delete temp table");
    map.insert(JET_errCannotDeleteSystemTable, "Illegal attempt to delete a system table");
    map.insert(JET_errCannotDeleteTemplateTable, "Illegal attempt to delete a template table");
    map.insert(JET_errExclusiveTableLockRequired, "Must have exclusive lock on table.");
    map.insert(JET_errFixedDDL, "DDL operations prohibited on this table");
    map.insert(JET_errFixedInheritedDDL, "On a derived table, DDL operations are prohibited on inherited portion of DDL");
    map.insert(JET_errCannotNestDDL, "Nesting of hierarchical DDL is not currently supported.");
    map.insert(JET_errDDLNotInheritable, "Tried to inherit DDL from a table not marked as a template table.");
    map.insert(JET_wrnTableInUseBySystem, "System cleanup has a cursor open on the table");
    map.insert(JET_errInvalidSettings, "System parameters were set improperly");
    map.insert(JET_errClientRequestToStopJetService, "Client has requested stop service");
    map.insert(JET_errCannotAddFixedVarColumnToDerivedTable, "Template table was created with NoFixedVarColumnsInDerivedTables");
    map.insert(JET_errIndexCantBuild, "Index build failed");
    map.insert(JET_errIndexHasPrimary, "Primary index already defined");
    map.insert(JET_errIndexDuplicate, "Index is already defined");
    map.insert(JET_errIndexNotFound, "No such index");
    map.insert(JET_errIndexMustStay, "Cannot delete clustered index");
    map.insert(JET_errIndexInvalidDef, "Illegal index definition");
    map.insert(JET_errInvalidCreateIndex, "Invalid create index description");
    map.insert(JET_errTooManyOpenIndexes, "Out of index description blocks");
    map.insert(JET_errMultiValuedIndexViolation, "Non-unique inter-record index keys generated for a multivalued index");
    map.insert(JET_errIndexBuildCorrupted, "Failed to build a secondary index that properly reflects primary index");
    map.insert(JET_errPrimaryIndexCorrupted, "Primary index is corrupt. The database must be defragmented or the table deleted.");
    map.insert(JET_errSecondaryIndexCorrupted, "Secondary index is corrupt. The database must be defragmented or the affected index must be deleted. If the corrupt index is over Unicode text, a likely cause a sort-order change.");
    map.insert(JET_wrnCorruptIndexDeleted, "Out of date index removed");
    map.insert(JET_errInvalidIndexId, "Illegal index id");
    map.insert(JET_wrnPrimaryIndexOutOfDate, "The Primary index is created with an incompatible OS sort version. The table can not be safely modified.");
    map.insert(JET_wrnSecondaryIndexOutOfDate, "One or more Secondary index is created with an incompatible OS sort version. Any index over Unicode text should be deleted.");
    map.insert(JET_errIndexTuplesSecondaryIndexOnly, "tuple index can only be on a secondary index");
    map.insert(JET_errIndexTuplesTooManyColumns, "tuple index may only have eleven columns in the index");
    map.insert(JET_errIndexTuplesOneColumnOnly, "OBSOLETE");
    map.insert(JET_errIndexTuplesNonUniqueOnly, "tuple index must be a non-unique index");
    map.insert(JET_errIndexTuplesTextBinaryColumnsOnly, "tuple index must be on a text/binary column");
    map.insert(JET_errIndexTuplesTextColumnsOnly, "OBSOLETE");
    map.insert(JET_errIndexTuplesVarSegMacNotAllowed, "tuple index does not allow setting cbVarSegMac");
    map.insert(JET_errIndexTuplesInvalidLimits, "invalid min/max tuple length or max characters to index specified");
    map.insert(JET_errIndexTuplesCannotRetrieveFromIndex, "cannot call RetrieveColumn() with RetrieveFromIndex on a tuple index");
    map.insert(JET_errIndexTuplesKeyTooSmall, "specified key does not meet minimum tuple length");
    map.insert(JET_errInvalidLVChunkSize, "Specified LV chunk size is not supported");
    map.insert(JET_errColumnCannotBeEncrypted, "Only JET_coltypLongText and JET_coltypLongBinary columns can be encrypted");
    map.insert(JET_errCannotIndexOnEncryptedColumn, "Cannot index encrypted column");
    map.insert(JET_errColumnLong, "Column value is long");
    map.insert(JET_errColumnNoChunk, "No such chunk in long value");
    map.insert(JET_errColumnDoesNotFit, "Field will not fit in record");
    map.insert(JET_errNullInvalid, "Null not valid");
    map.insert(JET_errColumnIndexed, "Column indexed, cannot delete");
    map.insert(JET_errColumnTooBig, "Field length is greater than maximum");
    map.insert(JET_errColumnNotFound, "No such column");
    map.insert(JET_errColumnDuplicate, "Field is already defined");
    map.insert(JET_errMultiValuedColumnMustBeTagged, "Attempted to create a multi-valued column, but column was not Tagged");
    map.insert(JET_errColumnRedundant, "Second autoincrement or version column");
    map.insert(JET_errInvalidColumnType, "Invalid column data type");
    map.insert(JET_wrnColumnMaxTruncated, "Max length too big, truncated");
    map.insert(JET_errTaggedNotNULL, "No non-NULL tagged columns");
    map.insert(JET_errNoCurrentIndex, "Invalid w/o a current index");
    map.insert(JET_errKeyIsMade, "The key is completely made");
    map.insert(JET_errBadColumnId, "Column Id Incorrect");
    map.insert(JET_errBadItagSequence, "Bad itagSequence for tagged column");
    map.insert(JET_errColumnInRelationship, "Cannot delete, column participates in relationship");
    map.insert(JET_wrnCopyLongValue, "Single instance column bursted");
    map.insert(JET_errCannotBeTagged, "AutoIncrement and Version cannot be tagged");
    map.insert(JET_errDefaultValueTooBig, "Default value exceeds maximum size");
    map.insert(JET_errMultiValuedDuplicate, "Duplicate detected on a unique multi-valued column");
    map.insert(JET_errLVCorrupted, "Corruption encountered in long-value tree");
    map.insert(JET_errMultiValuedDuplicateAfterTruncation, "Duplicate detected on a unique multi-valued column after data was normalized, and normalizing truncated the data before comparison");
    map.insert(JET_errDerivedColumnCorruption, "Invalid column in derived table");
    map.insert(JET_errInvalidPlaceholderColumn, "Tried to convert column to a primary index placeholder, but column doesn't meet necessary criteria");
    map.insert(JET_wrnColumnSkipped, "Column value(s) not returned because the corresponding column id or itagSequence requested for enumeration was null");
    map.insert(JET_wrnColumnNotLocal, "Column value(s) not returned because they could not be reconstructed from the data at hand");
    map.insert(JET_wrnColumnMoreTags, "Column values exist that were not requested for enumeration");
    map.insert(JET_wrnColumnTruncated, "Column value truncated at the requested size limit during enumeration");
    map.insert(JET_wrnColumnPresent, "Column values exist but were not returned by request");
    map.insert(JET_wrnColumnSingleValue, "Column value returned in JET_COLUMNENUM as a result of JET_bitEnumerateCompressOutput");
    map.insert(JET_wrnColumnDefault, "Column value(s) not returned because they were set to their default value(s) and JET_bitEnumerateIgnoreDefault was specified");
    map.insert(JET_errColumnCannotBeCompressed, "Only JET_coltypLongText and JET_coltypLongBinary columns can be compressed");
    map.insert(JET_wrnColumnNotInRecord, "Column value(s) not returned because they could not be reconstructed from the data in the record");
    map.insert(JET_errColumnNoEncryptionKey, "Cannot retrieve/set encrypted column without an encryption key");
    map.insert(JET_errRecordNotFound, "The key was not found");
    map.insert(JET_errRecordNoCopy, "No working buffer");
    map.insert(JET_errNoCurrentRecord, "Currency not on a record");
    map.insert(JET_errRecordPrimaryChanged, "Primary key may not change");
    map.insert(JET_errKeyDuplicate, "Illegal duplicate key");
    map.insert(JET_errAlreadyPrepared, "Attempted to update record when record update was already in progress");
    map.insert(JET_errKeyNotMade, "No call to JetMakeKey");
    map.insert(JET_errUpdateNotPrepared, "No call to JetPrepareUpdate");
    map.insert(JET_wrnDataHasChanged, "Data has changed");
    map.insert(JET_errDataHasChanged, "Data has changed, operation aborted");
    map.insert(JET_wrnKeyChanged, "Moved to new key");
    map.insert(JET_errLanguageNotSupported, "Windows installation does not support language");
    map.insert(JET_errDecompressionFailed, "Internal error: data could not be decompressed");
    map.insert(JET_errUpdateMustVersion, "No version updates only for uncommitted tables");
    map.insert(JET_errDecryptionFailed, "Data could not be decrypted");
    map.insert(JET_errTooManySorts, "Too many sort processes");
    map.insert(JET_errInvalidOnSort, "Invalid operation on Sort");
    map.insert(JET_errTempFileOpenError, "Temp file could not be opened");
    map.insert(JET_errTooManyAttachedDatabases, "Too many open databases");
    map.insert(JET_errDiskFull, "No space left on disk");
    map.insert(JET_errPermissionDenied, "Permission denied");
    map.insert(JET_errFileNotFound, "File not found");
    map.insert(JET_errFileInvalidType, "Invalid file type");
    map.insert(JET_wrnFileOpenReadOnly, "Database file is read only");
    map.insert(JET_errAfterInitialization, "Cannot Restore after init.");
    map.insert(JET_errLogCorrupted, "Logs could not be interpreted");
    map.insert(JET_errInvalidOperation, "Invalid operation");
    map.insert(JET_errAccessDenied, "Access denied");
    map.insert(JET_wrnIdleFull, "Idle registry full");
    map.insert(JET_errTooManySplits, "Infinite split");
    map.insert(JET_errSessionSharingViolation, "Multiple threads are using the same session");
    map.insert(JET_errEntryPointNotFound, "An entry point in a DLL we require could not be found");
    map.insert(JET_errSessionContextAlreadySet, "Specified session already has a session context set");
    map.insert(JET_errSessionContextNotSetByThisThread, "Tried to reset session context, but current thread did not orignally set the session context");
    map.insert(JET_errSessionInUse, "Tried to terminate session in use");
    map.insert(JET_errRecordFormatConversionFailed, "Internal error during dynamic record format conversion");
    map.insert(JET_errOneDatabasePerSession, "Just one open user database per session is allowed (JET_paramOneDatabasePerSession)");
    map.insert(JET_errRollbackError, "error during rollback");
    map.insert(JET_errFlushMapVersionUnsupported, "The version of the persisted flush map is not supported by this version of the engine.");
    map.insert(JET_errFlushMapDatabaseMismatch, "The persisted flush map and the database do not match.");
    map.insert(JET_errFlushMapUnrecoverable, "The persisted flush map cannot be reconstructed.");
    map.insert(JET_wrnDefragAlreadyRunning, "Online defrag already running on specified database");
    map.insert(JET_wrnDefragNotRunning, "Online defrag not running on specified database");
    map.insert(JET_errDatabaseAlreadyRunningMaintenance, "The operation did not complete successfully because the database is already running maintenance on specified database");
    map.insert(JET_wrnCallbackNotRegistered, "Unregistered a non-existant callback function");
    map.insert(JET_errCallbackFailed, "A callback failed");
    map.insert(JET_errCallbackNotResolved, "A callback function could not be found");
    map.insert(JET_errSpaceHintsInvalid, "An element of the JET space hints structure was not correct or actionable.");
    map.insert(JET_errOSSnapshotInvalidSequence, "OS Shadow copy API used in an invalid sequence");
    map.insert(JET_errOSSnapshotTimeOut, "OS Shadow copy ended with time-out");
    map.insert(JET_errOSSnapshotNotAllowed, "OS Shadow copy not allowed (backup or recovery in progress)");
    map.insert(JET_errOSSnapshotInvalidSnapId, "invalid JET_OSSNAPID");
    map.insert(JET_errLSCallbackNotSpecified, "Attempted to use Local Storage without a callback function being specified");
    map.insert(JET_errLSAlreadySet, "Attempted to set Local Storage for an object which already had it set");
    map.insert(JET_errLSNotSet, "Attempted to retrieve Local Storage from an object which didn't have it set");
    map.insert(JET_errFileIOSparse, "an I/O was issued to a location that was sparse");
    map.insert(JET_errFileIOBeyondEOF, "a read was issued to a location beyond EOF (writes will expand the file)");
    map.insert(JET_errFileIOAbort, "instructs the JET_ABORTRETRYFAILCALLBACK caller to abort the specified I/O");
    map.insert(JET_errFileIORetry, "instructs the JET_ABORTRETRYFAILCALLBACK caller to retry the specified I/O");
    map.insert(JET_errFileIOFail, "instructs the JET_ABORTRETRYFAILCALLBACK caller to fail the specified I/O");
    map.insert(JET_errFileCompressed, "read/write access is not supported on compressed files");
}
