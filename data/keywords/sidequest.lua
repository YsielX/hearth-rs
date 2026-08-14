return {
    api_version = 1, module_type = "keyword", id = "sidequest", name = "Sidequest",
    rules = { enters_secret_zone = function(ctx, self, current) return true end },
}
