pub const RESPONSE_404: &str = r##"
<html>
<header>
    <title>404 Not Found</title>
    <style>
        body {
            font-family: sans-serif;
            background-color: #f0f0f0;
            color: #333;
            margin: 0;
            padding: 0;
        }
        h1 {
            background-color: #333;
            color: #f0f0f0;
            padding: 10px;
            margin: 0;
        }
        p {
            padding: 10px;
            margin: 0;
        }
        .spacer {
            height: 1px;
            background-color: #333;
            width: 100%;
        }
    </style>
</header>
<body>
    <h1>"#%PATH%#" was not found</h1>
    <br>
    <p>The requested page was not found on the web server or is inaccessible to you.</p>
    <br>
    <div class="spacer"></div>
    <p>mowserver - #%VERSION%#</p>
</body>
</html>
"##;

pub const RESPONSE_INVALID: &str = r#"
<html>
<header>
    <title>Invalid Response</title>
    <style>
        body {
            font-family: sans-serif;
            background-color: #f0f0f0;
            color: #333;
            margin: 0;
            padding: 0;
        }
        h1 {
            background-color: #333;
            color: #f0f0f0;
            padding: 10px;
            margin: 0;
        }
        p {
            padding: 10px;
            margin: 0;
        }
        .spacer {
            height: 1px;
            background-color: #333;
            width: 100%;
        }
    </style>
</header>
<body>
    <h1>Invalid Request</h1>
    <br>
    <p>The request we received contained garbage. Please don't do that.</p>
    <br>
    <div class="spacer"></div>
    <p>mowserver - #%VERSION%#</p>
</body>
</html>
"#;

pub const RESPONSE_403: &str = r##"
<html>
<header>
    <title>403 Permssion Denied</title>
    <style>
        body {
            font-family: sans-serif;
            background-color: #f0f0f0;
            color: #333;
            margin: 0;
            padding: 0;
        }
        h1 {
            background-color: #333;
            color: #f0f0f0;
            padding: 10px;
            margin: 0;
        }
        p {
            padding: 10px;
            margin: 0;
        }
        .spacer {
            height: 1px;
            background-color: #333;
            width: 100%;
        }
    </style>
</header>
<body>
    <h1>"#%PATH%#" - Permission Denied</h1>
    <br>
    <p>You're not the type of person whom can see what you asked for. Turn away now!</p>
    <br>
    <div class="spacer"></div>
    <p>mowserver - #%VERSION%#</p>
</body>
</html>
"##;

pub const RESPONSE_SHUTDOWN: &str = r#"
<html>
<header>
    <title>MOWDOWN</title>
    <style>
        body {
            font-family: sans-serif;
            background-color: #f0f0f0;
            color: #333;
            margin: 0;
            padding: 0;
        }
        h1 {
            background-color: #333;
            color: #f0f0f0;
            padding: 10px;
            margin: 0;
        }
        p {
            padding: 10px;
            margin: 0;
        }
        .spacer {
            height: 1px;
            background-color: #333;
            width: 100%;
        }
    </style>
</header>
<body>
    <h1>MowServer is shutting down</h1>
    <br>
    <p>Your request made the mowserver mow down. I hope you got what you wanted. This is the last time you'll hear from me (Or you got load balanced / cached).</p>
    <br>
    <div class="spacer"></div>
    <p>mowserver - #%VERSION%#</p>
</body>
</html>
"#;
