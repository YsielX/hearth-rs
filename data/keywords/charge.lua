return {
    api_version = 1,
    module_type = "keyword",
    id = "charge",
    name = "Charge",
    rules = {
        ready_on_summon = function(ctx, self, current, other)
            return true
        end,
    },
}
