return {
    api_version = 1,
    module_type = "keyword",
    id = "tradeable",
    name = "Tradeable",
    rules = {
        can_trade = function(ctx, self, current)
            return true
        end,
    },
}
