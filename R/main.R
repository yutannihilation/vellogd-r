#' @export
vellogd <- function(filename = "Rplot%03d.png", width = 480, height = 480) {
  vellogd_impl(filename, as.numeric(width), as.numeric(height))
}
