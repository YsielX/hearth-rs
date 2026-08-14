return {
    api_version = 1,
    id = "ICC_099",
    name = "Ticking Abomination",
    text = "<b>Deathrattle:</b> Deal 5 damage to your minions.",
    set = "ICECROWN",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 5,
    health = 6,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords or {}) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant then targets[#targets + 1] = minion end
        end
        ctx:damage_all(targets, 5)
    end,
}
