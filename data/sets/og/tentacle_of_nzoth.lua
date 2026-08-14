return {
    api_version = 1, id = "OG_151", name = "Tentacle of N'Zoth",
    text = "<b>Deathrattle:</b> Deal 1 damage to all minions.", set = "OG",
    type = "minion", rarity = "common", cost = 1, attack = 1, health = 1,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant then targets[#targets + 1] = minion end
        end
        if #targets > 0 then ctx:damage_all(targets, 1) end
    end,
}
