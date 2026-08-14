return {
    api_version = 1,
    module_type = "keyword",
    id = "end_of_turn_repeater",
    name = "End Of Turn Repeater",
    rules = {
        end_of_turn_repetitions = function(ctx, self, current)
            return math.max(current, 2)
        end,
    },
}
