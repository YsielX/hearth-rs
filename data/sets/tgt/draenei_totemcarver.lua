return {
    api_version = 1, id = "AT_047", name = "Draenei Totemcarver",
    text = "<b>Battlecry:</b> Gain +1/+1 for each friendly Totem.", set = "TGT", type = "minion",
    class = "shaman", rarity = "rare", cost = 4, attack = 4, health = 5,
    tags = { "draenei" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local count = 0
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            for _, tag in ipairs(ctx:card_definition(ctx:entity(minion).card_id).tags or {}) do
                if tag == "totem" or tag == "all" then count = count + 1 break end
            end
        end
        if count > 0 then cardlib.effects.buff(ctx, self, count, count) end
    end,
}
