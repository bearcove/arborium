// --------------------------------------------------------------------------------------
// Utilities for working with network, downloading resources with specified headers etc.
// --------------------------------------------------------------------------------------

namespace FSharp.Data

open System
open System.Globalization
open System.IO
open System.Net
open System.Text
open System.Text.RegularExpressions
open System.Threading
open System.Reflection
open System.Runtime.CompilerServices
open System.Runtime.InteropServices

/// The method to use in an HTTP request
module HttpMethod =

    // RFC 2626 specifies 8 methods

    /// Request information about the communication options available on the request/response chain identified by the URI
    let Options = "OPTIONS"

    /// Retrieve whatever information (in the form of an entity) is identified by the URI
    let Get = "GET"

    /// Identical to GET except that the server MUST NOT return a message-body in the response
    let Head = "HEAD"

    /// Requests that the server accepts the entity enclosed in the request as a
    /// new subordinate of the resource identified by the Request-URI in the Request-Line
    let Post = "POST"

    /// Requests that the enclosed entity be stored under the supplied Request-URI
    let Put = "PUT"

    /// Requests that the origin server deletes the resource identified by the Request-URI
    let Delete = "DELETE"

    /// Used to invoke a remote, application-layer loop- back of the request message
    let Trace = "TRACE"

    /// Reserved for use with a proxy that can dynamically switch to being a tunnel
    let Connect = "CONNECT"

    // RFC 4918 (WebDAV) adds 7 methods

    /// Retrieves properties defined on the resource identified by the request URI
    let PropFind = "PROPFIND"

    /// Processes instructions specified in the request body to set and/or remove properties defined on the resource identified by the request URI
    let PropPatch = "PROPPATCH"

    /// Creates a new collection resource at the location specified by the Request URI
    let MkCol = "MKCOL"

    /// Creates a duplicate of the source resource, identified by the Request-URI, in the destination resource, identified by the URI in the Destination header
    let Copy = "COPY"

    /// Logical equivalent of a copy, followed by consistency maintenance processing, followed by a delete of the source where all three actions are performed atomically
    let Move = "MOVE"

    /// Used to take out a lock of any access type on the resource identified by the request URI.
    let Lock = "LOCK"

    /// Removes the lock identified by the lock token from the request URI, and all other resources included in the lock
    let Unlock = "UNLOCK"

    // RFC 5789 adds one more

    /// Requests that the origin server applies partial modifications contained in the entity enclosed in the request to the resource identified by the request URI
    let Patch = "PATCH"

/// Headers that can be sent in an HTTP request
module HttpRequestHeaders =
    /// Content-Types that are acceptable for the response
    let Accept (contentType: string) = "Accept", contentType

    /// Character sets that are acceptable
    let AcceptCharset (characterSets: string) = "Accept-Charset", characterSets

    /// Acceptable version in time
    let AcceptDatetime (dateTime: DateTime) =
        "Accept-Datetime", dateTime.ToString("R", CultureInfo.InvariantCulture)

    /// List of acceptable encodings. See HTTP compression.
    let AcceptEncoding (encoding: string) = "Accept-Encoding", encoding

    /// List of acceptable human languages for response
    let AcceptLanguage (language: string) = "Accept-Language", language

    /// The Allow header, which specifies the set of HTTP methods supported.
    let Allow (methods: string) = "Allow", methods

    /// Authentication credentials for HTTP authentication
    let Authorization (credentials: string) = "Authorization", credentials

    /// Authentication header using Basic Auth encoding
    let BasicAuth (username: string) (password: string) =
        let base64Encode (s: string) =
            let bytes = Encoding.UTF8.GetBytes(s)
            Convert.ToBase64String(bytes)

        sprintf "%s:%s" username password
        |> base64Encode
        |> sprintf "Basic %s"
        |> Authorization

    /// Used to specify directives that MUST be obeyed by all caching mechanisms along the request/response chain
    let CacheControl (control: string) = "Cache-Control", control

    /// What type of connection the user-agent would prefer
    let Connection (connection: string) = "Connection", connection

    /// Describes the placement of the content. Valid dispositions are: inline, attachment, form-data
    let ContentDisposition (placement: string, name: string option, fileName: string option) =
        let namePart =
            match name with
            | Some n -> sprintf "; name=\"%s\"" n
            | None -> ""

        let fileNamePart =
            match fileName with
            | Some n -> sprintf "; filename=\"%s\"" n
            | None -> ""

        "Content-Disposition", sprintf "%s%s%s" placement namePart fileNamePart

    /// The type of encoding used on the data
    let ContentEncoding (encoding: string) = "Content-Encoding", encoding

    /// The language the content is in
    let ContentLanguage (language: string) = "Content-Language", language

    /// An alternate location for the returned data
    let ContentLocation (location: string) = "Content-Location", location

    /// A Base64-encoded binary MD5 sum of the content of the request body
    let ContentMD5 (md5sum: string) = "Content-MD5", md5sum

    /// Where in a full body message this partial message belongs
    let ContentRange (range: string) = "Content-Range", range

    /// The MIME type of the body of the request (used with POST and PUT requests)
    let ContentType (contentType: string) = "Content-Type", contentType

    /// The MIME type of the body of the request (used with POST and PUT requests) with an explicit encoding
    let ContentTypeWithEncoding (contentType, charset: Encoding) =
