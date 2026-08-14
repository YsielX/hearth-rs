return {
    api_version = 1,
    module_type = "keyword",
    id = "conditional_charge",
    name = "Conditional Charge",
    rules = {
        ready_on_summon = function(ctx, self, current, other)
            local player = ctx:player(ctx:controller(self))
            return current or player.weapon ~= nil
        end,
    },
}
