local card = {
    api_version = 1, id = "CFM_751", name = "Abyssal Enforcer",
    text = "<b>Battlecry:</b> Deal 3 damage to all other characters.", set = "GANGS",
    type = "minion", class = "warlock", rarity = "common", cost = 7,
    attack = 6, health = 6, tags = { "demon" }, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local targets = {}
    for _, entity in ipairs(ctx:characters()) do if entity ~= self then targets[#targets + 1] = entity end end
    if #targets > 0 then ctx:damage_all(targets, 3) end
end
return card
