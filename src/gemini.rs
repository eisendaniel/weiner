/*Gemini Protocol and Text functions
*  IS responsible for requests
* IS responsible for handling gemini protocol status/reponses
* NOT responsible for state (history, current page, etc)
* NOT responsible for presentation (gui)
*/

use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

enum Status {
    Internal, //lib defined catch status (<10)

    _Input,           //10-19
    _Success,         //20-29
    Redirect(String), //30-39
    _TempFail,        //40-49
    _PermFail,        //50-59
    _ClientCert,      //60-69
}

fn request(abs_uri: &url::Url) -> Result<Vec<u8>, Status> {
    let host = abs_uri.host_str().ok_or(Status::Internal)?;
    let url = format!("{}:1965", host);

    let mut builder = TlsConnector::builder();
    builder.danger_accept_invalid_certs(true);
    let connector = builder.build().map_err(|_error| Status::Internal)?;

    let stream = TcpStream::connect(&url).map_err(|_error| Status::Internal)?;
    let mut mstream = connector
        .connect(host, stream)
        .map_err(|_error| Status::Internal)?;

    let request = format!("{}\r\n", abs_uri);

    mstream
        .write_all(request.as_bytes())
        .map_err(|_error| Status::Internal)?;
    let mut raw_response = vec![];
    mstream
        .read_to_end(&mut raw_response)
        .map_err(|_error| Status::Internal)?;

    let end_of_header = raw_response
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or(Status::Internal)?;

    let content = raw_response.split_off(end_of_header);
    let status_str = String::from_utf8_lossy(&raw_response).into_owned();

    let mut iter = status_str.split(' ');
    match iter.next() {
        Some("20") => Ok(content),
        Some("30") | Some("31") => Err(Status::Redirect(iter.next().unwrap().to_owned())), //todo! handle nothing
        _ => Ok(raw_response),
    }
}

/*
* WILL early return on error with Optional Content and URL fetched from
* DO burden caller with submitting valid request or recieve None.
* DO NOT burden caller with Results or handling gemini status codes
*/
pub fn fetch(url_str: &str) -> Option<(String, Vec<u8>)> {
    // !todo[0]
    let mut abs_uri = url::Url::parse(url_str).ok()?;

    loop {
        match request(&abs_uri) {
            Ok(content) => return Some((format!("{}", abs_uri.as_str()), content)),
            Err(status) => match status {
                Status::Internal => return None,
                Status::_Input => todo!(),
                Status::_Success => todo!(),
                Status::Redirect(to_str) => abs_uri = url::Url::parse(&to_str).ok()?,
                Status::_TempFail => todo!(),
                Status::_PermFail => todo!(),
                Status::_ClientCert => todo!(),
            },
        }
    }
}
