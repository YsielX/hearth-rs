return {
    api_version = 1, id = "CFM_667", name = "Bomb Squad",
    text = "[x]<b>Battlecry:</b> Deal 5 damage\nto an enemy minion.\n<b>Deathrattle:</b> Deal 5 damage\nto your hero.",
    set = "GANGS", type = "minion", rarity = "rare", cost = 5, attack = 2, health = 2,
    keywords = { "battlecry", "deathrattle" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
        end
        return result
    end,
    on_battlecry = function(ctx, self, target) if target then ctx:damage(target, 5) end end,
    on_deathrattle = function(ctx, self) ctx:damage(ctx:player(ctx:controller(self)).hero, 5) end,
}
