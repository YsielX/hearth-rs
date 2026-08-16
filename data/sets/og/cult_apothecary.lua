return {
    api_version = 1, id = "OG_295", name = "Cult Apothecary",
    text = "<b>Battlecry:</b> For each enemy minion, restore #2 Health to your hero.", set = "OG",
    type = "minion", rarity = "common", cost = 4, attack = 4, health = 4,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        local count = 0
        for _, entity in ipairs(ctx:board(ctx:opponent(player))) do
            local view = ctx:entity(entity)
            local dormant = false
            for _, keyword in ipairs(view.keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if view.type == "minion" and not dormant then count = count + 1 end
        end
        if count > 0 then cardlib.effects.heal(ctx, ctx:player(player).hero, count * 2) end
    end,
}
