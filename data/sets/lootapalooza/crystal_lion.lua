local function recruit(ctx, entity) return ctx:entity(entity).card_id == "CS2_101t" end
return {
    api_version = 1, id = "LOOT_313", name = "Crystal Lion",
    text = "<b>Divine Shield</b>\nCosts (1) less for each Silver Hand Recruit you control.",
    set = "LOOTAPALOOZA", type = "minion", class = "paladin", rarity = "rare",
    cost = 6, attack = 5, health = 5, tags = { "elemental", "beast" },
    keywords = { "divine_shield" },
    auras = {{ active_zones = { "hand", "deck" }, cost = function(ctx, self)
        local count = 0
        for _, minion in ipairs(ctx:friendly_minions(self)) do if recruit(ctx, minion) then count = count + 1 end end
        return -count
    end, targets = function(ctx, self) return { self } end }},
}
