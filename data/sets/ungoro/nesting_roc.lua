return {
    api_version = 1, id = "UNG_801", name = "Nesting Roc",
    text = "<b>Battlecry:</b> If you control at least 2 other minions, gain <b>Taunt</b>.",
    set = "UNGORO", type = "minion", rarity = "common", cost = 5, attack = 4, health = 7,
    tags = { "beast" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local others = 0
        for _, entity in ipairs(ctx:board(ctx:controller(self))) do
            if entity ~= self and ctx:entity(entity).type == "minion" then
                local is_dormant = false
                for _, keyword in ipairs(ctx:entity(entity).keywords) do
                    if keyword == "dormant" then is_dormant = true end
                end
                if not is_dormant then others = others + 1 end
            end
        end
        if others >= 2 then ctx:grant_keyword(self, "taunt") end
    end,
}
