resource "local_file" "dirty" {
  filename = "dirty.txt"
  content  = var.undefined_variable
}
