local card = {
    api_version = 1, id = "CFM_855", name = "Defias Cleaner",
    text = "<b>Battlecry:</b> <b>Silence</b> a minion with <b>Deathrattle</b>.",
    set = "GANGS", type = "minion", rarity = "epic", cost = 6, attack = 5,
    health = 7, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx)
        local result = {}
        for _, entity in ipairs(ctx:minions()) do
            for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
                if keyword == "deathrattle" then result[#result + 1] = entity break end
            end
        end
        return result
    end,
}
function card.on_battlecry(ctx, self, target) if target then ctx:silence(target) end end
return card
