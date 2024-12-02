#' Open A 'Vello' Graphics Device.
#' 
#' @param filename The name of the output file.
#' @param width,height The dimensions of the device in pixel.
#' @export
vellogd <- function(filename = "Rplot%03d.png", width = 480, height = 480) {
  vellogd_impl(filename, as.numeric(width), as.numeric(height))
}

#' @name vellogd
#' @export
vellogd_with_server <- function(filename = "Rplot%03d.png", width = 480, height = 480) {
  server <- server_path()
  vellogd_with_server_impl(filename, as.numeric(width), as.numeric(height), server)
}

#' Render A Lottie Animation File.
#' 
#' @param filename The path of a lottie file.
draw_lottie <- function(filename) {
  if (!isTRUE(file.exists(filename))) {
    stop(filename, "does not exist!", call. = FALSE)
  }

  add_lottie_animation(filename)
}