local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "CFM_328",
    name = "Fight Promoter",
    text = "[x]<b>Battlecry:</b> If you control\na minion with 6 or more\n Health, draw two cards.",
    set = "GANGS",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 4,
    health = 4,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if not is_dormant(ctx, minion) and ctx:entity(minion).health >= 6 then
                ctx:draw(ctx:controller(self), 2)
                return
            end
        end
    end,
}
