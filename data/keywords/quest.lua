return {
    api_version = 1,
    module_type = "keyword",
    id = "quest",
    name = "Quest",
    rules = {
        starts_in_opening_hand = function(ctx, self, current, other)
            return true
        end,
        enters_secret_zone = function(ctx, self, current, other)
            return true
        end,
    },
}
