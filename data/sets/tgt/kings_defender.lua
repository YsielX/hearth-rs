return {
    api_version = 1, id = "AT_065", name = "King's Defender",
    text = "<b>Battlecry:</b> If you have a minion with <b>Taunt</b>, gain +1 Durability.",
    set = "TGT", type = "weapon", class = "warrior", rarity = "rare", cost = 3,
    attack = 3, health = 2, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "taunt" then cardlib.effects.buff(ctx, self, 0, 1) return end
            end
        end
    end,
}
