//! Single-source prose for routing local-file work into the model-visible tool surface.

/// Positive routing shared by the host guidance file and every fresh MCP connection.
pub(crate) const LOCAL_FILE_ROUTE_GUIDANCE: &str = concat!(
    "Use FastCtx file tools directly for local-file operations, including when a\n",
    "local reference is URI-shaped; pass the equivalent plain absolute filesystem path."
);

/// Shared path-shape contract used by every file tool's path field.
pub(crate) const LOCAL_PATH_INPUT_GUIDANCE: &str = concat!(
    "Plain absolute local filesystem path. When the source reference is URI-shaped, ",
    "use its equivalent local absolute path."
);

const READ_TOOL_DETAILS: &str = concat!(
    "Read one file (text, image, or PDF) or a batch of text files from the local\n",
    "filesystem. Text returns 1-based `N<tab>content` lines, as much of the file as\n",
    "the output budget holds. For several text files in one call, pass\n",
    "files=[{\"path\": ...}, ...] instead of file_path: one token budget, per-file\n",
    "problems reported inline without failing the batch, and a Partial note returns\n",
    "the exact files array for the next call. Images (PNG/JPG/GIF/WebP/BMP) are\n",
    "shown to you visually. PDFs return the selected pages' text layer or those\n",
    "pages rendered as images; image mode defaults to 4 pages. view=\"hex\" dumps\n",
    "any file's raw bytes. PDFs, images, and hex view are single-file only. Text\n",
    "output is always UTF-8; when auto-detection is not confident it returns an\n",
    "error listing candidate encodings instead of guessed text, so pass encoding\n",
    "only then. Text, PDF, and hex responses end with a Complete or Partial status\n",
    "— continue only with the exact parameters a Partial note provides."
);

pub(crate) fn local_path_description(context: &str) -> String {
    format!("{LOCAL_PATH_INPUT_GUIDANCE} {context}")
}

pub(crate) fn read_tool_description() -> String {
    format!(
        "{} {READ_TOOL_DETAILS}",
        LOCAL_FILE_ROUTE_GUIDANCE.replace('\n', " ")
    )
}

pub(crate) fn server_instructions(enable_shell: bool) -> String {
    let tools = if enable_shell {
        "Local-file tools: read, grep, glob, replace, plus POSIX-bash shell tools."
    } else {
        "Local-file tools: read, grep, glob, and replace."
    };
    format!("{tools} {}", LOCAL_FILE_ROUTE_GUIDANCE.replace('\n', " "))
}
