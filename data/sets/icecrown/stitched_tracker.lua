local card = {
    api_version = 1, id = "ICC_415", name = "Stitched Tracker",
    text = "<b>Battlecry:</b> <b>Discover</b> a copy of a minion in your deck.",
    set = "ICECROWN", type = "minion", class = "hunter", rarity = "common",
    cost = 3, attack = 2, health = 2, tags = { "undead" }, keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then
        ctx:discover_entities(ctx:controller(self), "Choose a minion to copy", candidates, 3, "stitched_tracker_chosen")
    end
end

function card.stitched_tracker_chosen(ctx, self, entity)
    ctx:give_copy(ctx:controller(self), entity)
end

return card
