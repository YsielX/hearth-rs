local function played_elemental_last_turn(ctx, player)
    for _, card_id in ipairs(ctx:cards_played_last_turn(player)) do
        for _, tag in ipairs(ctx:card_definition(card_id).tags or {}) do
            if tag == "elemental" or tag == "all" then return true end
        end
    end
    return false
end

return {
    api_version = 1, id = "UNG_070", name = "Tol'vir Stoneshaper",
    text = "[x]<b>Battlecry:</b> If you played an\nElemental last turn, gain\n <b>Taunt</b> and <b>Divine Shield</b>.",
    set = "UNGORO", type = "minion", rarity = "rare", cost = 4,
    attack = 3, health = 6, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        if played_elemental_last_turn(ctx, ctx:controller(self)) then
            ctx:grant_keyword(self, "taunt")
            ctx:grant_keyword(self, "divine_shield")
        end
    end,
}
