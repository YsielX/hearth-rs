local card = {
    api_version = 1, id = "GVG_074", name = "Kezan Mystic",
    text = "<b>Battlecry:</b> Take control of a random enemy <b>Secret</b>.", set = "GVG",
    type = "minion", rarity = "rare", cost = 4, attack = 4, health = 3,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local secrets = ctx:secrets(ctx:opponent(ctx:controller(self)))
    if #secrets > 0 then ctx:random_entity(secrets, "take_secret") end
end
function card.take_secret(ctx, self, secret)
    ctx:change_controller(secret, ctx:controller(self))
end
return card
