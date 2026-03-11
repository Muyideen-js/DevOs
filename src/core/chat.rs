
/// Build the prompt payload for Ollama, including selected file contexts.
pub fn build_prompt(
    user_message: &str,
    context_files: &[(String, String)], // (filename, content)
    terminal_output: Option<&str>,
) -> String {
    let mut prompt = String::new();

    if !context_files.is_empty() {
        prompt.push_str("Here are the relevant files:\n\n");
        for (name, content) in context_files {
            prompt.push_str(&format!("--- {} ---\n{}\n\n", name, content));
        }
    }

    if let Some(output) = terminal_output {
        prompt.push_str(&format!(
            "Latest terminal output:\n```\n{}\n```\n\n",
            output
        ));
    }

    prompt.push_str(user_message);
    prompt
}

/// Send a chat message to the Ollama API and get a response.
/// This is a blocking call — should be run on a background thread.
pub fn send_message_blocking(
    endpoint: &str,
    model: &str,
    user_message: &str,
    context_files: &[(String, String)],
    terminal_output: Option<&str>,
) -> Result<String, String> {
    let prompt = build_prompt(user_message, context_files, terminal_output);

    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "system": "You are a coding assistant embedded in a developer IDE called DevOS. \
                   When the user asks you to modify code, output your changes as a unified diff. \
                   Be concise and helpful.",
        "stream": false,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("Failed to connect to Ollama at {}: {}", endpoint, e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned status {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .map_err(|e| format!("Invalid JSON response: {}", e))?;

    json["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No 'response' field in Ollama output".to_string())
}

/// Test connection to Ollama by hitting the root endpoint.
pub fn test_connection(endpoint: &str) -> Result<bool, String> {
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    match client.get(&url).send() {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(e) => Err(format!("Cannot reach Ollama: {}", e)),
    }
}

/// Fallback response when Ollama is not available.
pub fn fallback_response(user_message: &str) -> String {
    format!(
        "⚠️ Ollama is not connected. I'm running in fallback mode.\n\n\
         Your question: \"{}\"\n\n\
         To enable AI features:\n\
         1. Install Ollama from https://ollama.ai\n\
         2. Run: ollama pull llama3.1:8b\n\
         3. Make sure Ollama is running (ollama serve)\n\
         4. Check Settings → Test Connection",
        user_message
    )
}
