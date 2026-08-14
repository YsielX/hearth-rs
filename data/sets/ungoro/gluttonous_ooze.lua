local card = {
    api_version = 1, id = "UNG_946", name = "Gluttonous Ooze",
    text = "<b>Battlecry:</b> Destroy your opponent's weapon and gain Armor equal to its Attack.",
    set = "UNGORO", type = "minion", rarity = "epic", cost = 3, attack = 3, health = 3,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local weapon = ctx:player(ctx:opponent(ctx:controller(self))).weapon
    if weapon then local amount = ctx:entity(weapon).attack; ctx:destroy(weapon); if amount > 0 then ctx:gain_armor(ctx:controller(self), amount) end end
end
return card
