return {
    api_version = 1, id = "OG_276", name = "Blood Warriors",
    text = "Add a copy of each damaged friendly minion to your hand.", set = "OG", type = "spell",
    class = "warrior", rarity = "epic", cost = 3,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local entity = ctx:entity(minion)
            if entity.damage > 0 then ctx:give_base_copy(player, minion) end
        end
    end,
}
