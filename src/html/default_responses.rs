pub const RESPONSE_404: &str = r#"
<html>
<header>
    <title>404 Not Found</title>
</header>
<body>
    <h1>#%PATH%# was not found</h1>
    <p>The requested page was not found on the web server or is inaccessible to you.</p>
    <p>mowserver - #%VERSION%#</p>
</body>
</html>
"#;

pub const RESPONSE_INVALID: &str = r#"
<html>
<header>
    <title>Invalid Response</title>
</header>
<body>
    <h1>Invalid Request</h1>
    <p>The request we received contained garbage. Please don't do that.</p>
    <p>mowserver - #%VERSION%#</p>
</body>
</html>
"#;