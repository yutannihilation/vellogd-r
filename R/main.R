vellogd_env <- new.env(parent = emptyenv())

#' @export
vellogd <- function(filename = "Rplot%03d.png", width = 480, height = 480) {
  cmd <- "./src/dep/target/release/vellogd-server.exe"
  args <- c(as.character(width), as.character(height))
  vellogd_env$process <- processx::process$new(cmd, args)

  vellogd_impl(filename, as.numeric(width), as.numeric(height))
}
