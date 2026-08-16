return {
    api_version = 1, id = "OG_120", name = "Anomalus",
    text = "<b>Deathrattle:</b> Deal 8 damage to all minions.", set = "OG",
    type = "minion", class = "mage", rarity = "legendary", cost = 8,
    attack = 8, health = 6, tags = { "elemental" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant then targets[#targets + 1] = minion end
        end
        if #targets > 0 then cardlib.effects.damage_all(ctx, targets, 8) end
    end,
}
