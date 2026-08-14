local card = {
    api_version = 1, id = "ICC_221", name = "Leeching Poison",
    text = "Give your weapon <b>Lifesteal</b> this turn.", set = "ICECROWN",
    type = "spell", class = "rogue", rarity = "common", spell_school = "nature", cost = 1,
    rules = { can_play = function(ctx, self, current)
        return current and ctx:player(ctx:controller(self)).weapon ~= nil
    end },
}

function card.on_play(ctx, self)
    local weapon = ctx:player(ctx:controller(self)).weapon
    if weapon ~= nil then ctx:grant_keyword_until_end_of_turn(weapon, "lifesteal") end
end

return card
