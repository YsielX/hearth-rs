return {
    api_version = 1,
    id = "CFM_617",
    name = "Celestial Dreamer",
    text = "[x]<b>Battlecry:</b> If you control a\nminion with 5 or more\nAttack, gain +2/+2.",
    set = "GANGS",
    type = "minion",
    class = "druid",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 3,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            local dormant = false
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if not dormant and ctx:entity(minion).attack >= 5 then
                ctx:buff(self, 2, 2)
                return
            end
        end
    end,
}
