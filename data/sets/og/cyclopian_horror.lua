return {
    api_version = 1, id = "OG_337", name = "Cyclopian Horror",
    text = "<b>Taunt</b>. <b>Battlecry:</b> Gain      +1 Health for each enemy minion.", set = "OG",
    type = "minion", rarity = "epic", cost = 4, attack = 3, health = 3,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        local count = 0
        for _, entity in ipairs(ctx:board(ctx:opponent(ctx:controller(self)))) do
            local view = ctx:entity(entity)
            local dormant = false
            for _, keyword in ipairs(view.keywords) do
                if keyword == "dormant" then dormant = true break end
            end
            if view.type == "minion" and not dormant then count = count + 1 end
        end
        if count > 0 then ctx:buff(self, 0, count) end
    end,
}
