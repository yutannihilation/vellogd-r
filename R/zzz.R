.onUnload <- function(libpath) {
  if (!is.null(vellogd_env$process) && vellogd_env$process$is_alive()) {
    vellogd_env$process$kill()
  }
}