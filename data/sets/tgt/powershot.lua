return {
    api_version = 1, id = "AT_056", name = "Powershot",
    text = "Deal $2 damage to a minion and the minions next to it.",
    set = "TGT", type = "spell", class = "hunter", rarity = "rare", cost = 3,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        local victims = { target }
        for _, adjacent in ipairs(ctx:adjacent_minions(target)) do
            victims[#victims + 1] = adjacent
        end
        ctx:damage_all(victims, 2)
    end,
}
