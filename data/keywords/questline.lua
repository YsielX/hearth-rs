return {
    api_version = 1, module_type = "keyword", id = "questline", name = "Questline",
    rules = {
        starts_in_opening_hand = function(ctx, self, current) return true end,
        enters_secret_zone = function(ctx, self, current) return true end,
    },
}
