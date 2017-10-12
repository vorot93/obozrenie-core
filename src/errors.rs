extern crate std;
extern crate serde_json;
extern crate error_chain;
extern crate futures;

error_chain! {
    foreign_links {
	StringConversionError(std::string::FromUtf8Error);
	IOError(std::io::Error);
	JSONError(serde_json::Error);
    }
    errors {
        NotImplementedError(t: ()) {
            description("not implemented"),
            display("requested operation is stubbed but will be implemented in the future"),
        }
        NullError(t: String) {
            description(""),
            display(""),
        }
        NoSuchGameError(t: String) {
            description(""),
            display(""),
        }
        GameExistsError(t: String) {
            description(""),
            display(""),
        }
        NotFoundError(t: String) {
            description(""),
            display(""),
        }
        DataParseError(t: String) {
            description(""),
            display(""),
        }
        InvalidConfStorageError(t: String) {
            description(""),
            display(""),
        }
        InvalidSettingKeyError(t: String) {
            description(""),
            display(""),
        }
        SettingTypeMismatchError(t: String) {
            description(""),
            display(""),
        }
        BackendError(t: String) {
            description(""),
            display(""),
        }
    }
}

pub type FResult<T> = futures::future::Future<Item = T, Error = Error>;
