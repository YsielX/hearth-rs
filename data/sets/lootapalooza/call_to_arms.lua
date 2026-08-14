return {
    api_version = 1,
    id = "LOOT_093",
    name = "Call to Arms",
    text = "[x]<b>Recruit</b> 3 minions that\n cost (2) or less.",
    set = "LOOTAPALOOZA",
    type = "spell",
    rarity = "epic",
    class = "paladin",
    cost = 4,
    keywords = { "recruit" },
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        local recruited = 0
        for _, entity in ipairs(ctx:deck(player)) do
            local card = ctx:entity(entity)
            if card.type == "minion" and card.cost <= 2 then
                ctx:recruit(player, entity)
                recruited = recruited + 1
                if recruited == 3 then return end
            end
        end
    end,
}
