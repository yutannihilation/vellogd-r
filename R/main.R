#' Open A 'Vello' Graphics Device.
#' 
#' @export
vellogd <- function(filename = "Rplot%03d.png", width = 480, height = 480) {
  vellogd_impl(filename, as.numeric(width), as.numeric(height))
}

#' Open A 'Vello' Graphics Device With Server.
#' 
#' @export
vellogd <- function(filename = "Rplot%03d.png", width = 480, height = 480) {
  server <- server_path()
  vellogd_with_server_impl(filename, as.numeric(width), as.numeric(height), server)
}
