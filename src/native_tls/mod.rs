// Derived from https://lib.rs/crates/native-tls.

#![allow(dead_code)]

use std::any::Any;
use std::error;
use std::fmt;
use std::io;
use std::result;

// moved to ../lib.rs
// #[macro_use]
// extern crate log;

#[path = "openssl.rs"]
mod imp;

/// A typedef of the result-type returned by many methods.
pub type Result<T> = result::Result<T, Error>;

/// An error returned from the TLS implementation.
pub struct Error(imp::Error);

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        error::Error::source(&self.0)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, fmt)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl From<imp::Error> for Error {
    fn from(err: imp::Error) -> Error {
        Error(err)
    }
}

/// A cryptographic identity.
///
/// An identity is an X509 certificate along with its corresponding private key and chain of certificates to a trusted
/// root.
#[derive(Clone)]
pub struct Identity(imp::Identity);

/*
impl Identity {
    /// Parses a DER-formatted PKCS #12 archive, using the specified password to decrypt the key.
    ///
    /// The archive should contain a leaf certificate and its private key, as well any intermediate
    /// certificates that should be sent to clients to allow them to build a chain to a trusted
    /// root. The chain certificates should be in order from the leaf certificate towards the root.
    ///
    /// PKCS #12 archives typically have the file extension `.p12` or `.pfx`, and can be created
    /// with the OpenSSL `pkcs12` tool:
    ///
    /// ```bash
    /// openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem -certfile chain_certs.pem
    /// ```
    pub fn from_pkcs12(der: &[u8], password: &str) -> Result<Identity> {
        let identity = imp::Identity::from_pkcs12(der, password)?;
        Ok(Identity(identity))
    }
}
*/

/// An X509 certificate.
#[derive(Clone)]
pub struct Certificate(imp::Certificate);

/*
impl Certificate {
    /// Parses a DER-formatted X509 certificate.
    pub fn from_der(der: &[u8]) -> Result<Certificate> {
        let cert = imp::Certificate::from_der(der)?;
        Ok(Certificate(cert))
    }

    /// Parses a PEM-formatted X509 certificate.
    pub fn from_pem(pem: &[u8]) -> Result<Certificate> {
        let cert = imp::Certificate::from_pem(pem)?;
        Ok(Certificate(cert))
    }

    /// Returns the DER-encoded representation of this certificate.
    pub fn to_der(&self) -> Result<Vec<u8>> {
        let der = self.0.to_der()?;
        Ok(der)
    }
}
*/

/// A TLS stream which has been interrupted midway through the handshake process.
pub struct MidHandshakeTlsStream<S>(imp::MidHandshakeTlsStream<S>);

impl<S> fmt::Debug for MidHandshakeTlsStream<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

/*
impl<S> MidHandshakeTlsStream<S> {
    /// Returns a shared reference to the inner stream.
    pub fn get_ref(&self) -> &S {
        self.0.get_ref()
    }

    /// Returns a mutable reference to the inner stream.
    pub fn get_mut(&mut self) -> &mut S {
        self.0.get_mut()
    }
}

impl<S> MidHandshakeTlsStream<S>
where
    S: io::Read + io::Write,
{
    /// Restarts the handshake process.
    ///
    /// If the handshake completes successfully then the negotiated stream is
    /// returned. If there is a problem, however, then an error is returned.
    /// Note that the error may not be fatal. For example if the underlying
    /// stream is an asynchronous one then `HandshakeError::WouldBlock` may
    /// just mean to wait for more I/O to happen later.
    pub fn handshake(self) -> result::Result<TlsStream<S>, HandshakeError<S>> {
        match self.0.handshake() {
            Ok(s) => Ok(TlsStream(s)),
            Err(e) => Err(e.into()),
        }
    }
}
*/

/// An error returned from `ClientBuilder::handshake`.
#[derive(Debug)]
pub enum HandshakeError<S> {
    /// A fatal error.
    Failure(Error),

    /// A stream interrupted midway through the handshake process due to a
    /// `WouldBlock` error.
    ///
    /// Note that this is not a fatal error and it should be safe to call
    /// `handshake` at a later time once the stream is ready to perform I/O
    /// again.
    WouldBlock(MidHandshakeTlsStream<S>),
}

impl<S> error::Error for HandshakeError<S>
where
    S: Any + fmt::Debug,
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            HandshakeError::Failure(ref e) => Some(e),
            HandshakeError::WouldBlock(_) => None,
        }
    }
}

impl<S> fmt::Display for HandshakeError<S>
where
    S: Any + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandshakeError::Failure(ref e) => fmt::Display::fmt(e, fmt),
            HandshakeError::WouldBlock(_) => fmt.write_str("the handshake process was interrupted"),
        }
    }
}

impl<S> From<imp::HandshakeError<S>> for HandshakeError<S> {
    fn from(e: imp::HandshakeError<S>) -> HandshakeError<S> {
        match e {
            imp::HandshakeError::Failure(e) => HandshakeError::Failure(Error(e)),
            imp::HandshakeError::WouldBlock(s) => {
                HandshakeError::WouldBlock(MidHandshakeTlsStream(s))
            }
        }
    }
}

/// SSL/TLS protocol versions.
#[derive(Debug, Copy, Clone)]
#[allow(dead_code, clippy::manual_non_exhaustive)]
pub enum Protocol {
    /// The SSL 3.0 protocol.
    ///
    /// # Warning
    ///
    /// SSL 3.0 has severe security flaws, and should not be used unless absolutely necessary. If
    /// you are not sure if you need to enable this protocol, you should not.
    Sslv3,
    /// The TLS 1.0 protocol.
    Tlsv10,
    /// The TLS 1.1 protocol.
    Tlsv11,
    /// The TLS 1.2 protocol.
    Tlsv12,
    #[doc(hidden)]
    __NonExhaustive,
}

/// A builder for `TlsConnector`s.
pub struct TlsConnectorBuilder {
    identity: Option<Identity>,
    min_protocol: Option<Protocol>,
    max_protocol: Option<Protocol>,
    root_certificates: Vec<Certificate>,
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
    use_sni: bool,
    disable_built_in_roots: bool,
}

impl TlsConnectorBuilder {
    /*
    /// Sets the identity to be used for client certificate authentication.
    pub fn identity(&mut self, identity: Identity) -> &mut TlsConnectorBuilder {
        self.identity = Some(identity);
        self
    }

    /// Sets the minimum supported protocol version.
    ///
    /// A value of `None` enables support for the oldest protocols supported by the implementation.
    ///
    /// Defaults to `Some(Protocol::Tlsv10)`.
    pub fn min_protocol_version(&mut self, protocol: Option<Protocol>) -> &mut TlsConnectorBuilder {
        self.min_protocol = protocol;
        self
    }

    /// Sets the maximum supported protocol version.
    ///
    /// A value of `None` enables support for the newest protocols supported by the implementation.
    ///
    /// Defaults to `None`.
    pub fn max_protocol_version(&mut self, protocol: Option<Protocol>) -> &mut TlsConnectorBuilder {
        self.max_protocol = protocol;
        self
    }

    /// Adds a certificate to the set of roots that the connector will trust.
    ///
    /// The connector will use the system's trust root by default. This method can be used to add
    /// to that set when communicating with servers not trusted by the system.
    ///
    /// Defaults to an empty set.
    pub fn add_root_certificate(&mut self, cert: Certificate) -> &mut TlsConnectorBuilder {
        self.root_certificates.push(cert);
        self
    }

    /// Controls the use of built-in system certificates during certificate validation.
    ///
    /// Defaults to `false` -- built-in system certs will be used.
    pub fn disable_built_in_roots(&mut self, disable: bool) -> &mut TlsConnectorBuilder {
        self.disable_built_in_roots = disable;
        self
    }

    /// Controls the use of certificate validation.
    ///
    /// Defaults to `false`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If invalid certificates are trusted, *any*
    /// certificate for *any* site will be trusted for use. This includes expired certificates. This introduces
    /// significant vulnerabilities, and should only be used as a last resort.
    pub fn danger_accept_invalid_certs(
        &mut self,
        accept_invalid_certs: bool,
    ) -> &mut TlsConnectorBuilder {
        self.accept_invalid_certs = accept_invalid_certs;
        self
    }

    /// Controls the use of Server Name Indication (SNI).
    ///
    /// Defaults to `true`.
    pub fn use_sni(&mut self, use_sni: bool) -> &mut TlsConnectorBuilder {
        self.use_sni = use_sni;
        self
    }

    /// Controls the use of hostname verification.
    ///
    /// Defaults to `false`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If invalid hostnames are trusted, *any* valid
    /// certificate for *any* site will be trusted for use. This introduces significant vulnerabilities, and should
    /// only be used as a last resort.
    pub fn danger_accept_invalid_hostnames(
        &mut self,
        accept_invalid_hostnames: bool,
    ) -> &mut TlsConnectorBuilder {
        self.accept_invalid_hostnames = accept_invalid_hostnames;
        self
    }
    */

    /// Creates a new `TlsConnector`.
    pub fn build(&self) -> Result<TlsConnector> {
        let connector = imp::TlsConnector::new(self)?;
        Ok(TlsConnector(connector))
    }
}

/// A builder for client-side TLS connections.
///
/// # Examples
///
/// ```rust,ignore
/// use native_tls::TlsConnector;
/// use std::io::{Read, Write};
/// use std::net::TcpStream;
///
/// let connector = TlsConnector::new().unwrap();
///
/// let stream = TcpStream::connect("google.com:443").unwrap();
/// let mut stream = connector.connect("google.com", stream).unwrap();
///
/// stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
/// let mut res = vec![];
/// stream.read_to_end(&mut res).unwrap();
/// println!("{}", String::from_utf8_lossy(&res));
/// ```
#[derive(Clone, Debug)]
pub struct TlsConnector(imp::TlsConnector);

impl TlsConnector {
    /// Returns a new connector with default settings.
    pub fn new() -> Result<TlsConnector> {
        TlsConnector::builder().build()
    }

    /// Returns a new builder for a `TlsConnector`.
    pub fn builder() -> TlsConnectorBuilder {
        TlsConnectorBuilder {
            identity: None,
            min_protocol: Some(Protocol::Tlsv10),
            max_protocol: None,
            root_certificates: vec![],
            use_sni: true,
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
            disable_built_in_roots: false,
        }
    }

    /// Initiates a TLS handshake.
    ///
    /// The provided domain will be used for both SNI and certificate hostname
    /// validation.
    ///
    /// If the socket is nonblocking and a `WouldBlock` error is returned during
    /// the handshake, a `HandshakeError::WouldBlock` error will be returned
    /// which can be used to restart the handshake when the socket is ready
    /// again.
    ///
    /// The domain is ignored if both SNI and hostname verification are
    /// disabled.
    pub fn connect<S>(
        &self,
        domain: &str,
        stream: S,
    ) -> result::Result<TlsStream<S>, HandshakeError<S>>
    where
        S: io::Read + io::Write,
    {
        let s = self.0.connect(domain, stream)?;
        Ok(TlsStream(s))
    }
}

/*
/// A builder for `TlsAcceptor`s.
pub struct TlsAcceptorBuilder {
    identity: Identity,
    min_protocol: Option<Protocol>,
    max_protocol: Option<Protocol>,
}

impl TlsAcceptorBuilder {
    /// Sets the minimum supported protocol version.
    ///
    /// A value of `None` enables support for the oldest protocols supported by the implementation.
    ///
    /// Defaults to `Some(Protocol::Tlsv10)`.
    pub fn min_protocol_version(&mut self, protocol: Option<Protocol>) -> &mut TlsAcceptorBuilder {
        self.min_protocol = protocol;
        self
    }

    /// Sets the maximum supported protocol version.
    ///
    /// A value of `None` enables support for the newest protocols supported by the implementation.
    ///
    /// Defaults to `None`.
    pub fn max_protocol_version(&mut self, protocol: Option<Protocol>) -> &mut TlsAcceptorBuilder {
        self.max_protocol = protocol;
        self
    }

    /// Creates a new `TlsAcceptor`.
    pub fn build(&self) -> Result<TlsAcceptor> {
        let acceptor = imp::TlsAcceptor::new(self)?;
        Ok(TlsAcceptor(acceptor))
    }
}
*/

/// A builder for server-side TLS connections.
///
/// # Examples
///
/// ```rust,ignore
/// use native_tls::{Identity, TlsAcceptor, TlsStream};
/// use std::fs::File;
/// use std::io::{Read};
/// use std::net::{TcpListener, TcpStream};
/// use std::sync::Arc;
/// use std::thread;
///
/// let mut file = File::open("identity.pfx").unwrap();
/// let mut identity = vec![];
/// file.read_to_end(&mut identity).unwrap();
/// let identity = Identity::from_pkcs12(&identity, "hunter2").unwrap();
///
/// let listener = TcpListener::bind("0.0.0.0:8443").unwrap();
/// let acceptor = TlsAcceptor::new(identity).unwrap();
/// let acceptor = Arc::new(acceptor);
///
/// fn handle_client(stream: TlsStream<TcpStream>) {
///     // ...
/// }
///
/// for stream in listener.incoming() {
///     match stream {
///         Ok(stream) => {
///             let acceptor = acceptor.clone();
///             thread::spawn(move || {
///                 let stream = acceptor.accept(stream).unwrap();
///                 handle_client(stream);
///             });
///         }
///         Err(e) => { /* connection failed */ }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct TlsAcceptor(imp::TlsAcceptor);

/*
impl TlsAcceptor {
    /// Creates a acceptor with default settings.
    ///
    /// The identity acts as the server's private key/certificate chain.
    pub fn new(identity: Identity) -> Result<TlsAcceptor> {
        TlsAcceptor::builder(identity).build()
    }

    /// Returns a new builder for a `TlsAcceptor`.
    ///
    /// The identity acts as the server's private key/certificate chain.
    pub fn builder(identity: Identity) -> TlsAcceptorBuilder {
        TlsAcceptorBuilder {
            identity,
            min_protocol: Some(Protocol::Tlsv10),
            max_protocol: None,
        }
    }

    /// Initiates a TLS handshake.
    ///
    /// If the socket is nonblocking and a `WouldBlock` error is returned during
    /// the handshake, a `HandshakeError::WouldBlock` error will be returned
    /// which can be used to restart the handshake when the socket is ready
    /// again.
    pub fn accept<S>(&self, stream: S) -> result::Result<TlsStream<S>, HandshakeError<S>>
    where
        S: io::Read + io::Write,
    {
        match self.0.accept(stream) {
            Ok(s) => Ok(TlsStream(s)),
            Err(e) => Err(e.into()),
        }
    }
}
*/

/// A stream managing a TLS session.
pub struct TlsStream<S>(imp::TlsStream<S>);

impl<S: fmt::Debug> fmt::Debug for TlsStream<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl<S> TlsStream<S> {
    /// Returns a shared reference to the inner stream.
    pub fn get_ref(&self) -> &S {
        self.0.get_ref()
    }

    /// Returns a mutable reference to the inner stream.
    #[allow(dead_code)]
    pub fn get_mut(&mut self) -> &mut S {
        self.0.get_mut()
    }
}

/*
impl<S: io::Read + io::Write> TlsStream<S> {
    /// Returns the number of bytes that can be read without resulting in any
    /// network calls.
    pub fn buffered_read_size(&self) -> Result<usize> {
        Ok(self.0.buffered_read_size()?)
    }

    /// Returns the peer's leaf certificate, if available.
    pub fn peer_certificate(&self) -> Result<Option<Certificate>> {
        Ok(self.0.peer_certificate()?.map(Certificate))
    }

    /// Returns the tls-server-end-point channel binding data as defined in [RFC 5929].
    ///
    /// [RFC 5929]: https://tools.ietf.org/html/rfc5929
    pub fn tls_server_end_point(&self) -> Result<Option<Vec<u8>>> {
        Ok(self.0.tls_server_end_point()?)
    }

    /// Shuts down the TLS session.
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.0.shutdown()?;
        Ok(())
    }
}
*/

impl<S: io::Read + io::Write> io::Read for TlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<S: io::Read + io::Write> io::Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

fn _check_kinds() {
    use std::net::TcpStream;

    fn is_sync<T: Sync>() {}
    fn is_send<T: Send>() {}
    is_sync::<Error>();
    is_send::<Error>();
    is_sync::<TlsConnectorBuilder>();
    is_send::<TlsConnectorBuilder>();
    is_sync::<TlsConnector>();
    is_send::<TlsConnector>();
    /*
    is_sync::<TlsAcceptorBuilder>();
    is_send::<TlsAcceptorBuilder>();
    */
    is_sync::<TlsAcceptor>();
    is_send::<TlsAcceptor>();
    is_sync::<TlsStream<TcpStream>>();
    is_send::<TlsStream<TcpStream>>();
    is_sync::<MidHandshakeTlsStream<TcpStream>>();
    is_send::<MidHandshakeTlsStream<TcpStream>>();
}
