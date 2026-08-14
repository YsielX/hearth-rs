local function friendly_pirates(ctx, self)
    local count = 0
    for _, entity in ipairs(ctx:board(ctx:controller(self))) do
        for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
            if tag == "pirate" or tag == "all" then count = count + 1 break end
        end
    end
    return count
end

return {
    api_version = 1, id = "AT_070", name = "Skycap'n Kragg",
    text = "<b>Charrrrrge</b>\nCosts (1) less for each friendly Pirate.",
    set = "TGT", type = "minion", rarity = "legendary", cost = 7, attack = 4, health = 6,
    tags = { "pirate" }, keywords = { "charge" },
    auras = {{
        active_zones = { "hand", "deck" },
        cost = function(ctx, self) return -friendly_pirates(ctx, self) end,
        targets = function(ctx, self) return { self } end,
    }},
}
