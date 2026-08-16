return {
    api_version = 1, id = "AT_063", name = "Acidmaw",
    text = "Whenever an enemy minion takes damage, destroy it.",
    set = "TGT", type = "minion", class = "hunter", rarity = "legendary",
    cost = 3, attack = 4, health = 2, tags = { "beast" },
    triggers = {
        {
            event = "damaged", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event)
                local target = ctx:entity(event.target)
                return target.type == "minion"
                    and target.controller ~= ctx:controller(self)
                    and event.amount > 0
            end,
            effect = function(ctx, self, event) cardlib.effects.destroy(ctx, event.target) end,
        },
    },
}
