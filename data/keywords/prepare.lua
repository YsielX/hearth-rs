return {
    api_version = 1, module_type = "keyword", id = "prepare", name = "Prepare",
    rules = {
        can_play = function(ctx, self, current)
            return current and ctx:get_data(self, "prepared_turn") ~= ctx:turn()
        end,
    },
    actions = {
        prepare = {
            active_zones = { "hand" },
            spend_all_mana = true,
            condition = function(ctx, self)
                return ctx:get_data(self, "prepared_turn") ~= ctx:turn()
            end,
            effect = function(ctx, self, spent, target)
                cardlib.effects.modify(ctx, self, {
                    stat = "cost", operation = "add", value = -(spent + 1),
                })
                ctx:set_data(self, "prepared_turn", ctx:turn())
            end,
        },
    },
}
