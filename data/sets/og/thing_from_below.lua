local function summoned_totems(ctx, self)
    local count = 0
    for _, card_id in ipairs(ctx:minions_summoned(ctx:controller(self))) do
        for _, tag in ipairs(ctx:card_definition(card_id).tags) do
            if tag == "totem" or tag == "all" then count = count + 1 break end
        end
    end
    return count
end

return {
    api_version = 1, id = "OG_028", name = "Thing from Below",
    text = "[x]<b>Taunt</b>\nCosts (1) less for each\nTotem you've summoned\nthis game.", set = "OG",
    type = "minion", class = "shaman", rarity = "rare", cost = 6, attack = 5, health = 5,
    keywords = { "taunt" }, auras = {{
        active_zones = { "hand", "deck" },
        cost = function(ctx, self) return -summoned_totems(ctx, self) end,
        targets = function(ctx, self) return { self } end,
    }},
}
