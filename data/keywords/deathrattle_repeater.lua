return {
    api_version = 1,
    module_type = "keyword",
    id = "deathrattle_repeater",
    name = "Deathrattle Repeater",
    rules = {
        deathrattle_repetitions = function(ctx, self, current)
            return math.max(current, 2)
        end,
    },
}
