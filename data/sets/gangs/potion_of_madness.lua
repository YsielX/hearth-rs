local card = {
    api_version = 1, id = "CFM_603", name = "Potion of Madness",
    text = "Gain control of an enemy minion with 2 or less Attack until end of turn.",
    set = "GANGS", type = "spell", class = "priest", rarity = "common", spell_school = "shadow",
    cost = 1, target_mode = "required",
}
function card.targets(ctx, self)
    local result = {}
    if #ctx:board(ctx:controller(self)) >= 7 then return result end
    for _, entity in ipairs(ctx:enemy_characters(self)) do
        local info = ctx:entity(entity)
        if info.type == "minion" and info.attack <= 2 then result[#result + 1] = entity end
    end
    return result
end
function card.on_play(ctx, self, target)
    ctx:change_controller_until_end_of_turn(target, ctx:controller(self))
    ctx:grant_keyword_until_end_of_turn(target, "charge")
end
return card
