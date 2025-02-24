/*Gemini Protocol and Text functions
*  IS responsible for requests
*  IS responsible for gemtext parsing (!TODO)
* NOT responsible for state (history, current page, etc)
* NOT responsible for presentation (gui)
*/

use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

/*
* early return on error with Optional Content
* burden caller with submitting valid request or recieve None.
*/

pub fn fetch(url_str: &str) -> Option<Vec<u8>> { // !todo[0]
    let link = url::Url::parse(url_str).ok()?;
    
    let host = link.host_str()?;
    let url = format!("{}:1965", host);
    
    
    let mut builder = TlsConnector::builder();
    builder.danger_accept_invalid_certs(true);
    let connector = builder.build().ok()?;
    
    let stream = TcpStream::connect(&url).ok()?;    
    let mut mstream = connector.connect(host, stream).unwrap();
    

    let request = format!("{}\r\n", link);

    mstream.write_all(request.as_bytes()).unwrap();
    let mut response = vec![];
    mstream.read_to_end(&mut response).unwrap();

    Some(response)
}
