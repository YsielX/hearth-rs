return {
    api_version = 1,
    id = "OG_222",
    name = "Rallying Blade",
    text = "<b>Battlecry:</b> Give +1/+1 to your minions with <b>Divine Shield</b>.",
    set = "OG",
    type = "weapon",
    class = "paladin",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 2,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local dormant = false
            local divine_shield = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true end
                if keyword == "divine_shield" then divine_shield = true end
            end
            if divine_shield and not dormant then cardlib.effects.buff(ctx, minion, 1, 1) end
        end
    end,
}
