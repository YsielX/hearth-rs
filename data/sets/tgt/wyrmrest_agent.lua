local function is_dragon(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "AT_116",
    name = "Wyrmrest Agent",
    text = "<b>Battlecry:</b> If you're holding a Dragon, gain +1 Attack and <b>Taunt</b>.",
    set = "TGT",
    type = "minion",
    class = "priest",
    rarity = "rare",
    cost = 2,
    attack = 1,
    health = 4,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
            if is_dragon(ctx, entity) then
                ctx:buff(self, 1, 0)
                ctx:grant_keyword(self, "taunt")
                return
            end
        end
    end,
}
