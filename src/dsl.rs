/// Creates Rust structure for new Ruby class
///
/// This macro does not define an actual Ruby class. It only creates structs for using
/// the class in Rust. To define the class in Ruby, use `Class` structure.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate ruru;
///
/// use ruru::{Class, RString, Object, VM};
///
/// class!(Greeter);
///
/// methods!(
///     Greeter,
///     itself,
///
///     fn anonymous_greeting() -> RString {
///         RString::new("Hello stranger!")
///     }
///
///     fn friendly_greeting(name: RString) -> RString {
///         let name = name
///             .map(|name| name.to_string())
///             .unwrap_or("Anonymous".to_string());
///
///         let greeting = format!("Hello dear {}!", name);
///
///         RString::new(&greeting)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Greeter", None).define(|itself| {
///         itself.def("anonymous_greeting", anonymous_greeting);
///         itself.def("friendly_greeting", friendly_greeting);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Greeter
///   def anonymous_greeting
///     'Hello stranger!'
///   end
///
///   def friendly_greeting(name)
///     default_name = 'Anonymous'
///
///     name = defaut_name unless name.is_a?(String)
///
///     "Hello dear #{name}"
///   end
/// end
/// ```
#[macro_export]
macro_rules! class {
    ($class: ident) => {
        #[derive(Debug, PartialEq)]
        pub struct $class {
            value: $crate::types::Value,
        }

        impl From<$crate::types::Value> for $class {
            fn from(value: $crate::types::Value) -> Self {
                $class { value: value }
            }
        }

        impl $crate::Object for $class {
            #[inline]
            fn value(&self) -> $crate::types::Value {
                self.value
            }
        }
    }
}

/// Creates unsafe callbacks for Ruby methods
///
/// This macro is unsafe, because:
///
///  - it uses automatic unsafe conversions for arguments
///     (no guarantee that Ruby objects match the types which you expect);
///  - no bound checks for the array of provided arguments
///     (no guarantee that all the expected arguments are provided);
///
/// That is why creating callbacks in unsafe way may cause panics.
///
/// Due to the same reasons unsafe callbacks are faster.
///
/// Use it when:
///
///  - you own the Ruby code which passes arguments to callback;
///  - you are sure that all the object has correct type;
///  - you are sure that all the required arguments are provided;
///  - Ruby code has a good test coverage.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate ruru;
///
/// use ruru::{Boolean, Class, Fixnum, Object, RString, VM};
///
/// // Creates `string_length_equals` functions
/// unsafe_methods!(
///     RString, // type of `self` object
///     itself, // name of `self` object which will be used in methods
///
///     fn string_length_equals(expected_length: Fixnum) -> Boolean {
///         let real_length = itself.to_string().len() as i64;
///
///         Boolean::new(expected_length.to_i64() == real_length)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("String").define(|itself| {
///         itself.def("length_equals?", string_length_equals);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class String
///   def blank?
///     # ...
///   end
///
///   def length_equals?(expected_length)
///     # ...
///   end
/// end
/// ```
#[macro_export]
macro_rules! unsafe_methods {
    (
        $itself_class: ty,
        $itself_name: ident,
        $(
            fn $method_name: ident
            ($($arg_name: ident: $arg_type: ty),*) -> $return_type: ty $body: block
        )*
    ) => {
        $(
            #[no_mangle]
            #[allow(unused_mut)]
            pub extern fn $method_name(argc: $crate::types::Argc,
                                       argv: *const $crate::AnyObject,
                                       mut $itself_name: $itself_class) -> $return_type {
                let _arguments = $crate::VM::parse_arguments(argc, argv);
                let mut _i = 0;

                $(
                    let $arg_name = unsafe {
                        <$crate::AnyObject as $crate::Object>
                            ::to::<$arg_type>(&_arguments[_i])
                    };

                    _i += 1;
                )*

                $body
            }
        )*
    }
}

/// Creates callbacks for Ruby methods
///
/// Unlike `unsafe_methods!`, this macro is safe, because:
///
///  - it uses safe conversions of arguments (`Object::try_convert_to()`);
///  - it checks if arguments are present;
///
/// Each argument will have type `Result<Object, Error>`.
///
/// For example, if you declare `number: Fixnum` in the method definition, it will have actual
/// type `number: Result<Fixnum, Error>`.
///
/// See examples below and docs for `Object::try_convert_to()` for more information.
///
/// # Examples
///
/// To launch a server in Rust, you plan to write a simple `Server` class
///
/// ```ruby
/// class Server
///   def start(address)
///     # ...
///   end
/// end
/// ```
///
/// The `address` must be `Hash` with the following structure:
///
/// ```ruby
/// {
///   host: 'localhost',
///   port: 8080,
/// }
/// ```
///
/// You want to extract port from it. Default port is `8080` in case when:
///
///  - `address` is not a `Hash`
///  - `address[:port]` is not present
///  - `address[:port]` is not a `Fixnum`
///
/// ```
/// #[macro_use]
/// extern crate ruru;
///
/// use ruru::{Class, Fixnum, Hash, NilClass, Object, Symbol, VM};
///
/// class!(Server);
///
/// methods!(
///     Server,
///     itself,
///
///     fn start(address: Hash) -> NilClass {
///         let default_port = 8080;
///
///         let port = address
///             .map(|hash| hash.at(Symbol::new("port")))
///             .and_then(|port| port.try_convert_to::<Fixnum>())
///             .map(|port| port.to_i64())
///             .unwrap_or(default_port);
///
///         // Start server...
///
///         NilClass::new()
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Server", None).define(|itself| {
///         itself.def("start", start);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Server
///   def start(address)
///     default_port = 8080
///
///     port =
///       if address.is_a?(Hash) && address[:port].is_a?(Fixnum)
///         address[:port]
///       else
///         default_port
///       end
///
///     # Start server...
///   end
/// end
/// ```
#[macro_export]
macro_rules! methods {
    (
        $itself_class: ty,
        $itself_name: ident,
        $(
            fn $method_name: ident
            ($($arg_name: ident: $arg_type: ty),*) -> $return_type: ident $body: block
        )*
    ) => {
        $(
            #[no_mangle]
            #[allow(unused_mut)]
            pub extern fn $method_name(argc: $crate::types::Argc,
                                       argv: *const $crate::AnyObject,
                                       mut $itself_name: $itself_class) -> $return_type {
                let _arguments = $crate::VM::parse_arguments(argc, argv);
                let mut _i = 0;

                $(
                    let $arg_name =
                        _arguments
                            .get(_i)
                            .ok_or({
                                $crate::result::Error::ArgumentError(
                                    format!(
                                        "Argument '{}: {}' not found for method '{}'",
                                        stringify!($arg_name),
                                        stringify!($arg_type),
                                        stringify!($method_name)
                                    )
                                )
                            }).and_then(|argument| {
                                <$crate::AnyObject as $crate::Object>
                                    ::try_convert_to::<$arg_type>(argument)
                            });

                    _i += 1;
                )*

                $body
            }
        )*
    }
}

/// Makes a Rust struct wrappable for Ruby objects.
///
/// **Note:** Currently to be able to use `wrappable_struct!` macro, you should include
/// `lazy_static` crate to the crate you are working on.
///
/// `Cargo.toml`
///
/// ```toml
/// lazy_static = "0.2.1" # the version is not a strict requirement
/// ```
///
/// Crate root `lib.rs` or `main.rs`
///
/// ```ignore
/// #[macro_use]
/// extern crate lazy_static;
/// ```
///
/// # Arguments
///
///  - `$struct_name` is name of the actual Rust struct. This structure has to be public (`pub`).
///
///  - `$wrapper` is a name for the structcure which will be created to wrap the `$struct_name`.
///
///     The wrapper will be created automatically by the macro.
///
///  - `$static_name` is a name for a static variable which will contain the wrapper.
///
///     The static variable will be created automatically by the macro.
///
///     This variable has to be passed to `wrap_data()` and `get_data()` functions (see examples).
///
///     Also, these variables describe the structure in general, but not some specific object.
///     So you should pass the same wrapper static variable when wrapping/getting data of the same
///     kind for different ruby objects.
///
///     For example,
///
///     ```ignore
///     server1.get_data(&*SERVER_WRAPPER);
///     server2.get_data(&*SERVER_WRAPPER); // <-- the same `SERVER_WRAPPER`
///     ```
///
/// # Examples
///
/// Wrap `Server` structs to `RubyServer` objects
///
/// ```
/// #[macro_use] extern crate ruru;
/// #[macro_use] extern crate lazy_static;
///
/// use ruru::{AnyObject, Class, Fixnum, Object, RString, VM};
///
/// // The structure which we want to wrap
/// pub struct Server {
///     host: String,
///     port: u16,
/// }
///
/// impl Server {
///     fn new(host: String, port: u16) -> Self {
///         Server {
///             host: host,
///             port: port,
///         }
///     }
///
///     fn host(&self) -> &str {
///         &self.host
///     }
///
///     fn port(&self) -> u16 {
///         self.port
///     }
/// }
///
/// wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
///
/// class!(RubyServer);
///
/// methods!(
///     RubyServer,
///     itself,
///
///     fn ruby_server_new(host: RString, port: Fixnum) -> AnyObject {
///         let server = Server::new(host.unwrap().to_string(),
///                                  port.unwrap().to_i64() as u16);
///
///         Class::from_existing("RubyServer").wrap_data(server, &*SERVER_WRAPPER)
///     }
///
///     fn ruby_server_host() -> RString {
///         let host = itself.get_data(&*SERVER_WRAPPER).host();
///
///         RString::new(host)
///     }
///
///     fn ruby_server_port() -> Fixnum {
///         let port = itself.get_data(&*SERVER_WRAPPER).port();
///
///         Fixnum::new(port as i64)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("RubyServer", None).define(|itself| {
///         itself.def_self("new", ruby_server_new);
///
///         itself.def("host", ruby_server_host);
///         itself.def("port", ruby_server_port);
///     });
/// }
/// ```
///
/// To use the `RubyServer` class in Ruby:
///
/// ```ruby
/// server = RubyServer.new("127.0.0.1", 3000)
///
/// server.host == "127.0.0.1"
/// server.port == 3000
/// ```
#[macro_export]
macro_rules! wrappable_struct {
    ($struct_name: ty, $wrapper: ident, $static_name: ident) => {
        pub struct $wrapper<T> {
            data_type: $crate::types::DataType,
            _marker: ::std::marker::PhantomData<T>,
        }

        lazy_static! {
            pub static ref $static_name: $wrapper<$struct_name> = $wrapper::new();
        }

        impl<T> $wrapper<T> {
            fn new() -> $wrapper<T> {
                let name = concat!("Ruru/", stringify!($struct_name));
                let name = $crate::util::str_to_cstring(name);
                let reserved_bytes: [*mut $crate::types::c_void; 2] = [::std::ptr::null_mut(); 2];

                let data_type = $crate::types::DataType {
                    wrap_struct_name: name.into_raw(),
                    parent: ::std::ptr::null(),
                    data: ::std::ptr::null_mut(),
                    flags: $crate::types::Value::from(0),

                    function: $crate::types::DataTypeFunction {
                        dmark: None,
                        dfree: Some($crate::typed_data::free::<T>),
                        dsize: None,
                        reserved: reserved_bytes,
                    },
                };

                $wrapper {
                    data_type: data_type,
                    _marker: ::std::marker::PhantomData,
                }
            }
        }

        unsafe impl<T> Sync for $wrapper<T> {}

        // Set constraint to be able to wrap and get data only for type `T`
        impl<T> $crate::typed_data::DataTypeWrapper<T> for $wrapper<T> {
            fn data_type(&self) -> &$crate::types::DataType {
                &self.data_type
            }
        }
    }
}
