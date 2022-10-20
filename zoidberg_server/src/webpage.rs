use chrono::Utc;
use zoidberg_lib::types::{Job, Worker};

// TODO: write nicer frontend
pub fn render(jobs: &[Job], workers: &[Worker]) -> String {
    let jobs_html: String = String::from("<table class=\"table is-hoverable\">")
        + "<thead><tr><th><td>ID</td><td>command</td><td>status</td></th></tr></thead><tbody>"
        + &jobs
            .iter()
            .map(|j| {
                format!(
                    "<tr><th></th><td>{}</td><td>{}</td><td>{}</td></tr>",
                    j.id, j.cmd, j.status
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
        + "</tbody></table>";

    let workers_html: String = String::from("<table class=\"table is-hoverable\">")
        + "<thead><tr><th><td>ID</td><td>last heartbeat</td></th></tr></thead><tbody>"
        + &workers
            .iter()
            .map(|w| {
                let ts = if let Some(ts) = w.last_heartbeat {
                    format!("{}", Utc::now().timestamp() - ts)
                } else {
                    String::from("")
                };
                format!("<tr><th></th><td>{}</td><td>{}</td></tr>", w.id, ts)
            })
            .collect::<Vec<String>>()
            .join("\n")
        + "</tbody></table>";

    let _debug_html = r#"<style>
      *:not(path):not(g) {{
        color:                    hsla(210, 100%, 100%, 0.9) !important;
        background:               hsla(210, 100%,  50%, 0.5) !important;
        outline:    solid 0.25rem hsla(210, 100%, 100%, 0.5) !important;

        box-shadow: none !important;
      }}
    </style>"#;
    let _debug_html = "";

    let page = format!(
        r#"
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Zoidberg</title>
    <link rel="icon" href="data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20viewBox='0%200%2016%2016'%3E%3Ctext%20x='0'%20y='14'%3E🦀%3C/text%3E%3C/svg%3E" type="image/svg+xml" />

    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css">
    {}
  </head>
  <body>
  <section class="section">
    <div class="container">
      <div class="columns">
        <div class="column">
          <div class="block">
            <h1 class="title">
              Jobs
            </h1>
            {}
          </div>
        </div>
        <div class="column">
          <div class="block">
            <h1 class="title">
              Workers
            </h1>
            {}
          </div>
        </div>
      </div>
    </div>
  </section>
  </body>
</html>
"#,
        _debug_html, jobs_html, workers_html
    );
    page
}
