local function holding_dragon(ctx, self)
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
            if tag == "dragon" or tag == "all" then return true end
        end
    end
    return false
end

return {
    api_version = 1, id = "AT_071", name = "Alexstrasza's Champion",
    text = "<b>Battlecry:</b> If you're holding a Dragon, gain +1 Attack and <b>Charge</b>.",
    set = "TGT", type = "minion", class = "warrior", rarity = "rare", cost = 2,
    attack = 2, health = 3, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        if holding_dragon(ctx, self) then cardlib.effects.buff(ctx, self, 1, 0); cardlib.effects.grant_keyword(ctx, self, "charge") end
    end,
}
