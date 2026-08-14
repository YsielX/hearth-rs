local function summoned_beasts(ctx, self)
    local count = 0
    for _, card_id in ipairs(ctx:minions_summoned(ctx:controller(self))) do
        for _, tag in ipairs(ctx:card_definition(card_id).tags) do
            if tag == "beast" or tag == "all" then count = count + 1 break end
        end
    end
    return count
end

return {
    api_version = 1, id = "AT_041", name = "Knight of the Wild",
    text = "Costs (1) less for each Beast you've summoned this game.",
    set = "TGT", type = "minion", class = "druid", rarity = "rare",
    cost = 7, attack = 6, health = 6,
    auras = {
        {
            active_zones = { "hand", "deck" },
            cost = function(ctx, self) return -summoned_beasts(ctx, self) end,
            targets = function(ctx, self) return { self } end,
        },
    },
}
