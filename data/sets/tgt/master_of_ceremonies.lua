return {
    api_version = 1, id = "AT_117", name = "Master of Ceremonies",
    text = "<b>Battlecry:</b> If you have a minion with <b>Spell Damage</b>, gain +2/+2.",
    set = "TGT", type = "minion", rarity = "epic", cost = 3, attack = 4, health = 2,
    keywords = { "battlecry" }, on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            for _, keyword in ipairs(ctx:entity(minion).keywords) do
                if keyword == "spell_damage" then cardlib.effects.buff(ctx, self, 2, 2) return end
            end
        end
    end,
}
