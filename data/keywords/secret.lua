return {
    api_version = 1, module_type = "keyword", id = "secret", name = "Secret",
    rules = {
        enters_secret_zone = function(ctx, self, current) return true end,
    },
}
